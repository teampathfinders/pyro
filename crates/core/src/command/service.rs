use std::{sync::{Arc, OnceLock, Weak}, time::Duration};

use anyhow::Context as _;
use dashmap::DashMap;
use parking_lot::RwLock;
use proto::bedrock::{AvailableCommands, Command, DynamicEnumAction, ParseResult, ParsedCommand, UpdateDynamicEnum};
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use util::{CowString, Joinable};

use crate::instance::Instance;

const SERVICE_TIMEOUT: Duration = Duration::from_millis(10);

/// Represents a single output message in the command service response.
#[derive(Debug)]
pub struct CommandOutput {
    /// Output of the command.
    pub message: CowString<'static>,
    // pub message: Cow<'static, str>
    /// Optional parameters used in the command output.
    // pub parameters: Vec<Cow<'static, str>>
    pub parameters: Vec<CowString<'static>>
}

/// The result of a command execution.
pub type ExecutionResult = Result<CommandOutput, CommandOutput>;

/// A request that can be sent to the command [`Service`].
#[derive(Debug)]
pub struct ServiceRequest {
    command: String,
    callback: oneshot::Sender<ExecutionResult>
}

// /// Contains data accessible to command handlers
// pub struct Context {
//     instance: Arc<Instance>
//     // level: Arc<crate::level::Service>,
//     // commands: Arc<crate::command::Service>
// }

pub type Context = Arc<Instance>;

/// A function that parses and executes a command.
pub trait CommandHandler: Send + Sync {
    /// Executes the command using this handler.
    /// This function also performs parsing of the input.
    fn call(&self, input: &str, ctx: &Context) -> ExecutionResult;
    /// Returns the syntactic structure of the command.
    fn structure(&self) -> &Command;
}

/// A handler that uses the built-in command parser.
pub struct HandlerImpl<F> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync
{
    handler: F,
    structure: Command,
}

impl<F> CommandHandler for HandlerImpl<F> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync
{
    fn call(&self, input: &str, ctx: &Context) -> ExecutionResult {
        // Parse command with default parser.
        let parsed = match ParsedCommand::default_parser(&self.structure, input) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Err(CommandOutput {
                    message: err.description,
                    parameters: Vec::new()
                })
            }
        };

        (self.handler)(parsed, ctx)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}

/// A handler that uses a custom user-provided parser.
pub struct ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync,
    P: Fn(&str, &Context) -> ParseResult + Send + Sync
{
    handler: F,
    parser: P,
    structure: Command
}

impl<F, P> CommandHandler for ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync,
    P: Fn(&str, &Context) -> ParseResult + Send + Sync
{
    fn call(&self, input: &str, ctx: &Context) -> ExecutionResult {
        // Parse command with a custom parser.
        let parsed = match (self.parser)(input, ctx) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Err(CommandOutput {
                    message: err.description,
                    parameters: Vec::new()
                })
            }
        };

        (self.handler)(parsed, ctx)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}


/// Service that manages command execution.
pub struct Service {
    dynamic_enums: DashMap<String, Vec<String>>,

    token: CancellationToken,
    handle: RwLock<Option<JoinHandle<()>>>,

    sender: mpsc::Sender<ServiceRequest>,
    instance: OnceLock<Weak<Instance>>,

    /// Up to date [`AvailableCommands`] packet that can be sent to new users.
    available: RwLock<AvailableCommands<'static>>,
    registry: DashMap<String, Arc<dyn CommandHandler>>
}

impl Service {
    /// Creates a new command service.
    pub fn new(token: CancellationToken) -> Arc<Service> {
        let (sender, receiver) = mpsc::channel(10);
        let service = Arc::new(Service {
            token, sender,
            handle: RwLock::new(None), 
            registry: DashMap::new(),
            dynamic_enums: DashMap::new(),
            available: RwLock::new(AvailableCommands::empty()),
            instance: OnceLock::new()
        });

        let clone = Arc::clone(&service);
        let handle = tokio::spawn(async move {
            clone.execution_job(receiver).await
        });

        *service.handle.write() = Some(handle);
        service
    }

    /// Sets the instance pointer of this service.
    /// 
    /// This is used to create contexts when calling command handlers.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.instance.set(Arc::downgrade(instance)).map_err(|_| anyhow::anyhow!("Instance was already set"))
    }   

    /// Returns the current [`AvailableCommands`] packet.
    pub(crate) fn available_commands(&self) -> AvailableCommands<'static> {
        self.available.read().clone()
    }

    /// Updates autocompletion entries for the given dynamic enum.
    /// 
    /// This function can only be used with enums that were marked as dynamic on creation.
    /// 
    /// This function returns an error if the dynamic enum does not exist or
    /// if sending the update packet to clients fails.
    pub fn update_dynamic_enum(&self, update: UpdateDynamicEnum) -> anyhow::Result<()> {
        let Some(mut denum) = self.dynamic_enums.get_mut(update.enum_id) else {
            anyhow::bail!("Dynamic enum does not exist")
        };

        match update.action {
            DynamicEnumAction::Add => {
                denum.extend_from_slice(update.options);
            }
            DynamicEnumAction::Set => {
                denum.clear();
                denum.extend_from_slice(update.options);
            }
            DynamicEnumAction::Remove => {
                denum.retain(|opt| {
                    !update.options.contains(opt)
                })
            }
        }

        self.instance().clients().broadcast(update)
    }

    /// Registers a raw handler with this service.
    pub fn register_handler(&self, handler: Arc<dyn CommandHandler>) {
        let structure = handler.structure();
        self.available.write().commands.push(structure.clone());

        for alias in &structure.aliases {
            self.registry.insert(alias.clone(), Arc::clone(&handler));
        }

        for overload in &structure.overloads {
            for parameter in &overload.parameters {
                let Some(denum) = &parameter.command_enum else { continue };
                if denum.dynamic {
                    self.dynamic_enums.insert(denum.enum_id.clone(), denum.options.clone());
                }
            }
        }

        self.registry.insert(structure.name.clone(), handler);
    }

    /// Registers a new command with the default syntax parser. 
    /// 
    /// ## Arguments
    /// 
    /// * `structure` - This is the syntactic structure of the command and is what the game uses
    /// for autocompletion. This grammar is also what is used by the server's command parser to
    /// parse and validate the command.
    /// 
    /// * `handler` - This is the function that is ran by the service when your command is executed
    /// by a client. 
    pub fn register<F>(&self, structure: Command, handler: F) 
    where
        F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync + 'static
    {   
        let handler = Arc::new(HandlerImpl {
            handler, structure
        });
        
        self.register_handler(handler);
    }

    /// Registers a new command with a custom parser.
    pub fn register_with_parser<F, P>(&self, structure: Command, handler: F, parser: P) 
    where
        F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync + 'static,
        P: Fn(&str, &Context) -> ParseResult + Send + Sync + 'static
    {
        let handler = Arc::new(ParserHandlerImpl {
            handler, structure, parser
        });
        
        self.register_handler(handler);
    }

    /// Removes a command from the registry and returns its handler.
    /// 
    /// This function does not accept command aliases, you should use the original name of the command.
    pub fn unregister<S: AsRef<str>>(&self, name: S) -> Option<Arc<dyn CommandHandler>> {
        self.registry.remove(name.as_ref()).map(|(_, v)| v)
    }

    /// Request execution of a command.
    /// 
    /// This method will return a receiver that will receive the output when the command has been executed.
    /// Execution of the command might not happen within the same tick.
    pub async fn request(&self, command: String) -> anyhow::Result<oneshot::Receiver<ExecutionResult>> {
        let (sender, receiver) = oneshot::channel();
        let request = ServiceRequest { command, callback: sender };

        self.sender.send_timeout(request, SERVICE_TIMEOUT).await.context("Command service request timed out")?;

        Ok(receiver)
    }

    /// Returns the instance that owns this service.
    fn instance(&self) -> Arc<Instance> {
        // This will not panic because the instance field is initialised before the first command can be executed.
        #[allow(clippy::unwrap_used)]
        self.instance.get().unwrap().upgrade().unwrap()
    }

    /// Parses the syntactic structure of a command before sending it off to a custom handler.
    fn execute_handler(&self, command: &str, ctx: &Context) -> ExecutionResult {
        let command_name = {
            let mut split = command.split(' ');
            let first = split
                .next()
                .ok_or_else(|| {
                    CommandOutput {
                        message: "Expected command name after /".into(),
                        parameters: Vec::new()
                    }
                })?;

            // Get rid of slash in front of name.
            let mut chars = first.chars();
            chars.next();
            chars.as_str()
        };
        
        let Some(handler) = self.registry.get(command_name) else {
            return Err(CommandOutput {
                message: format!("Unknown command {command_name}. Make sure the command exists and you have permission to use it.").into(),
                parameters: Vec::new()
            })
        };
        
        handler.call(command, ctx)
    }

    /// Runs the service execution job.
    async fn execution_job(self: Arc<Service>, mut receiver: mpsc::Receiver<ServiceRequest>) {
        loop {
            tokio::select! {
                opt = receiver.recv() => {
                    let Some(request) = opt else {
                        tracing::error!("Command service lost all references, this is a bug. Shutting down the service");
                        break
                    };

                    let clone = Arc::clone(&self);
                    tokio::spawn(async move {
                        let Some(ctx) = clone.instance.get() else {
                            tracing::error!("Command service instance was not set");
                            return;
                        };

                        let Some(ctx) = ctx.upgrade() else {
                            tracing::error!("Attempt to create command context failed: instance has been dropped");
                            return;
                        };

                        let result = clone.execute_handler(&request.command, &ctx);
                        // Error can be ignored because it only occurs if the receiver does not exist anymore.
                        let _: Result<(), ExecutionResult> = request.callback.send(result);
                    });
                }
                _ = self.token.cancelled() => {
                    // Stop accepting requests.
                    receiver.close();
                    break
                }   
            }
        }

        tracing::info!("Command service closed");
    }
}

impl Joinable for Service {
    async fn join(&self) -> anyhow::Result<()> {
        let handle = self.handle.write().take();
        match handle {
            Some(handle) => Ok(handle.await?),
            None => anyhow::bail!("This command service has already been joined.")
        }
    }
}