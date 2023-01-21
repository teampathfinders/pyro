use std::net::{IpAddr, SocketAddr};

use bytes::{Buf, BufMut, BytesMut};

use crate::error::VexResult;
use crate::raknet::{Frame, FrameBatch, OrderChannel, Reliability};
use crate::raknet::packets::{ConnectedPing, Decodable, NewIncomingConnection};
use crate::services::{IPV4_LOCAL_ADDR, IPV6_LOCAL_ADDR};
use crate::util::{ReadExtensions, WriteExtensions};

#[test]
fn frame_decode() {
    // Sequence number 1
    // ReliableOrdered
    let frame = [
        0x80, 0x01, 0x0, 0x0, // New Incoming Connection
        0x60, 0x5, 0xd0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x13, 0x4, 0x80, 0xff, 0xff, 0xfe, 0x4a,
        0xbc, 0x6, 0x17, 0x0, 0xe2, 0x23, 0x0, 0x0, 0x0, 0x0, 0xfe, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0xa9, 0xce, 0xda, 0x6c, 0x1f, 0xb8, 0x6, 0xdb, 0xe, 0x0, 0x0, 0x0,
        // Empty IPv4 addresses
        0x4, 0x3f, 0x57, 0xcb, 0xab, 0xe2, 0x23, 0x4, 0xf5, 0xf5, 0xe6, 0x36, 0xe2, 0x23, 0x4, 0xff,
        0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff,
        0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0,
        0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4,
        0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff,
        0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff,
        0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0,
        0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x4, 0xff,
        0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x87, 0xe4, 0x4a, 0x0, 0x0, 0x48, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x87, 0xe4, 0x4a,
        // Connected Ping
        0x60, 0x0, 0x40, 0x2, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, // Compressed packet?
        0xfe, 0x6, 0xc1, 0x1, 0x0, 0x0, 0x2, 0x30,
    ];

    let decoded = FrameBatch::decode(BytesMut::from(frame.as_ref())).unwrap();
    let nic = NewIncomingConnection::decode(decoded.frames[0].body.clone());
    dbg!(nic);
    // let ping = ConnectedPing::decode(decoded.frames[1].body.clone());

    // println!("{nic:#?} {ping:#?}");
}

#[test]
fn read_write_u24_le() {
    let mut buffer = BytesMut::new();
    buffer.put_u24_le(125); // Test first byte only
    buffer.put_u24_le(50250); // Test first two bytes
    buffer.put_u24_le(1097359); // Test all bytes

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_u24_le(), 125);
    assert_eq!(buffer.get_u24_le(), 50250);
    assert_eq!(buffer.get_u24_le(), 1097359);
}

#[test]
fn read_write_addr() -> VexResult<()> {
    let ipv4_test = SocketAddr::new(IpAddr::V4(IPV4_LOCAL_ADDR), 19132);
    let ipv6_test = SocketAddr::new(IpAddr::V6(IPV6_LOCAL_ADDR), 19133);

    let mut buffer = BytesMut::new();
    buffer.put_addr(ipv4_test); // Test IPv4
    buffer.put_addr(ipv6_test); // Test IPv6

    let mut buffer = buffer.freeze();
    assert_eq!(buffer.get_addr()?, ipv4_test);
    assert_eq!(buffer.get_addr()?, ipv6_test);
    Ok(())
}

#[test]
fn order_channel() {
    let mut test_frame = Frame::default();
    let mut channel = OrderChannel::new();

    test_frame.order_index = 0;
    assert!(channel.insert(test_frame.clone()).is_some());

    test_frame.order_index = 2;
    assert!(channel.insert(test_frame.clone()).is_none());

    test_frame.order_index = 1;
    let output = channel.insert(test_frame).unwrap();

    assert_eq!(output.len(), 2);
    assert_eq!(output[0].order_index, 1);
    assert_eq!(output[1].order_index, 2);
}
