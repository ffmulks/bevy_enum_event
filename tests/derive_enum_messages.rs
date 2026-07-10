//! Tests for the `EnumMessage` derive macro (buffered messages)
//! These tests verify that generated message types work correctly with
//! Bevy's MessageWriter/MessageReader system.

use bevy::prelude::*;
use bevy_enum_event::EnumMessage;

// ============================================================================
// Basic Message Tests
// ============================================================================

// Test unit variants
#[derive(EnumMessage, Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
enum UnitMessage {
    Start,
    Stop,
    Pause,
}

#[test]
fn test_unit_message_variants() {
    let start = unit_message::Start;
    let stop = unit_message::Stop;
    let pause = unit_message::Pause;

    // Verify they can be created and are distinct types
    assert_eq!(std::mem::size_of_val(&start), 0);
    assert_eq!(std::mem::size_of_val(&stop), 0);
    assert_eq!(std::mem::size_of_val(&pause), 0);
}

// Test tuple variants
#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum TupleMessage {
    Data(u32),
    Text(String),
    Pair(u32, String),
}

#[test]
fn test_tuple_message_variants() {
    let data = tuple_message::Data(42);
    assert_eq!(data.0, 42);

    let text = tuple_message::Text("hello".to_string());
    assert_eq!(text.0, "hello");

    let pair = tuple_message::Pair(123, "world".to_string());
    assert_eq!(pair.0, 123);
    assert_eq!(pair.1, "world");
}

// Test named field variants
#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum NamedMessage {
    Request { id: u32, payload: String },
    Response { id: u32, data: Vec<u8> },
    Empty,
}

#[test]
fn test_named_message_variants() {
    let request = named_message::Request {
        id: 1,
        payload: "test".to_string(),
    };
    assert_eq!(request.id, 1);
    assert_eq!(request.payload, "test");

    let response = named_message::Response {
        id: 2,
        data: vec![1, 2, 3],
    };
    assert_eq!(response.id, 2);
    assert_eq!(response.data, vec![1, 2, 3]);

    let empty = named_message::Empty;
    assert_eq!(std::mem::size_of_val(&empty), 0);
}

// Test mixed variants
#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum MixedMessage {
    Unit,
    Tuple(String),
    Named { value: i32 },
}

#[test]
fn test_mixed_message_variants() {
    let unit = mixed_message::Unit;
    assert_eq!(std::mem::size_of_val(&unit), 0);

    let tuple = mixed_message::Tuple("test".to_string());
    assert_eq!(tuple.0, "test");

    let named = mixed_message::Named { value: 42 };
    assert_eq!(named.value, 42);
}

// ============================================================================
// Deref Tests
// ============================================================================

#[cfg(feature = "deref")]
#[test]
fn test_deref_tuple_message() {
    #[derive(EnumMessage, Clone)]
    #[allow(dead_code)]
    enum DerefTupleMessage {
        Value(String),
    }

    let mut val = deref_tuple_message::Value("test".to_string());

    // Test Deref
    let s: &String = &val;
    assert_eq!(s, "test");

    // Test DerefMut
    let s_mut: &mut String = &mut val;
    s_mut.push_str("_modified");
    assert_eq!(val.0, "test_modified");
}

#[cfg(feature = "deref")]
#[test]
fn test_deref_named_message() {
    #[derive(EnumMessage, Clone)]
    #[allow(dead_code)]
    enum DerefNamedMessage {
        Value { data: String },
    }

    let mut val = deref_named_message::Value {
        data: "test".to_string(),
    };

    // Test Deref
    let s: &String = &val;
    assert_eq!(s, "test");

    // Test DerefMut
    let s_mut: &mut String = &mut val;
    s_mut.push_str("_modified");
    assert_eq!(val.data, "test_modified");
}

#[cfg(feature = "deref")]
#[test]
fn test_multi_field_deref_with_attribute() {
    #[derive(EnumMessage, Clone)]
    #[allow(dead_code)]
    enum MultiFieldDerefMessage {
        Tuple(#[enum_event(deref)] String, i32),
        Named {
            #[enum_event(deref)]
            value: String,
            other: i32,
        },
    }

    let mut tuple = multi_field_deref_message::Tuple("tuple".to_string(), 7);
    let tuple_ref: &String = &tuple;
    assert_eq!(tuple_ref, "tuple");

    let tuple_ref_mut: &mut String = &mut tuple;
    tuple_ref_mut.push_str("_updated");
    assert_eq!(tuple.0, "tuple_updated");

    let mut named = multi_field_deref_message::Named {
        value: "named".to_string(),
        other: 9,
    };
    let named_ref: &String = &named;
    assert_eq!(named_ref, "named");

    let named_ref_mut: &mut String = &mut named;
    named_ref_mut.push_str("_updated");
    assert_eq!(named.value, "named_updated");
    assert_eq!(named.other, 9);
}

// ============================================================================
// Generic Support Tests
// ============================================================================

#[test]
fn test_generic_message_support() {
    #[derive(EnumMessage, Clone, Debug)]
    #[allow(dead_code)]
    enum GenericMessage<T>
    where
        T: Clone + std::fmt::Debug,
    {
        Owned(T),
        Pair(T, u32),
        Unit,
    }

    #[derive(EnumMessage, Clone, Copy, Debug)]
    #[allow(dead_code)]
    enum BorrowedMessage<'a> {
        Reference(&'a i32),
        Unit,
    }

    let value = String::from("hello");
    let owned = generic_message::Owned(value.clone());
    assert_eq!(owned.0, value);

    let pair = generic_message::Pair(value.clone(), 7);
    assert_eq!(pair.0, value);
    assert_eq!(pair.1, 7);

    let _unit = generic_message::Unit::<String>::default();

    let data = 42;
    let reference = borrowed_message::Reference(&data);
    #[cfg(feature = "deref")]
    assert_eq!(**reference, 42);
    #[cfg(not(feature = "deref"))]
    assert_eq!(*reference.0, 42);

    let _borrowed_unit = borrowed_message::Unit::default();
}

// ============================================================================
// Integration with Bevy MessageWriter/MessageReader
// ============================================================================

#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum TestNetworkMessage {
    Connected { player_id: u32 },
    Disconnected { player_id: u32, reason: String },
    DataReceived(Vec<u8>),
}

#[derive(Resource, Default)]
struct ReceivedMessages {
    connections: Vec<u32>,
    // Mirrors the `Disconnected` message variant; not exercised by this test.
    #[allow(dead_code)]
    disconnections: Vec<(u32, String)>,
    data_packets: Vec<Vec<u8>>,
}

#[derive(Resource, Default)]
struct MessagesSent(bool);

fn write_connected_message(
    mut writer: MessageWriter<test_network_message::Connected>,
    mut sent: ResMut<MessagesSent>,
) {
    if sent.0 {
        return;
    }
    sent.0 = true;
    writer.write(test_network_message::Connected { player_id: 1 });
    writer.write(test_network_message::Connected { player_id: 2 });
}

fn read_connected_messages(
    mut reader: MessageReader<test_network_message::Connected>,
    mut received: ResMut<ReceivedMessages>,
) {
    for msg in reader.read() {
        received.connections.push(msg.player_id);
    }
}

#[derive(Resource, Default)]
struct DataMessagesSent(bool);

fn write_data_message(
    mut writer: MessageWriter<test_network_message::DataReceived>,
    mut sent: ResMut<DataMessagesSent>,
) {
    if sent.0 {
        return;
    }
    sent.0 = true;
    writer.write(test_network_message::DataReceived(vec![1, 2, 3]));
    writer.write(test_network_message::DataReceived(vec![4, 5, 6]));
}

fn read_data_messages(
    mut reader: MessageReader<test_network_message::DataReceived>,
    mut received: ResMut<ReceivedMessages>,
) {
    for msg in reader.read() {
        received.data_packets.push(msg.0.clone());
    }
}

#[test]
fn test_message_writer_reader_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ReceivedMessages>();
    app.init_resource::<MessagesSent>();

    // Register the message types
    app.add_message::<test_network_message::Connected>();
    app.add_message::<test_network_message::DataReceived>();

    // Add systems that write and read messages
    app.add_systems(
        Update,
        (
            write_connected_message,
            read_connected_messages.after(write_connected_message),
        ),
    );

    // Run one frame to write messages
    app.update();
    // Run another frame to read them
    app.update();

    // Verify messages were received
    let received = app.world().resource::<ReceivedMessages>();
    assert_eq!(received.connections.len(), 2, "Should have 2 connections");
    assert!(received.connections.contains(&1), "Should contain player 1");
    assert!(received.connections.contains(&2), "Should contain player 2");
}

#[test]
fn test_tuple_message_writer_reader() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ReceivedMessages>();
    app.init_resource::<DataMessagesSent>();

    // Register the message type
    app.add_message::<test_network_message::DataReceived>();

    // Add systems
    app.add_systems(
        Update,
        (
            write_data_message,
            read_data_messages.after(write_data_message),
        ),
    );

    // Run frames
    app.update();
    app.update();

    // Verify
    let received = app.world().resource::<ReceivedMessages>();
    assert_eq!(received.data_packets.len(), 2, "Should have 2 data packets");
    assert_eq!(received.data_packets[0], vec![1, 2, 3]);
    assert_eq!(received.data_packets[1], vec![4, 5, 6]);
}

// ============================================================================
// Message with Multiple Readers
// ============================================================================

// The enum only exists to generate the `broadcast_message` module used below.
#[derive(EnumMessage, Clone, Debug)]
#[allow(dead_code)]
enum BroadcastMessage {
    Announcement(String),
}

#[derive(Resource, Default)]
struct Reader1Count(usize);

#[derive(Resource, Default)]
struct Reader2Count(usize);

#[derive(Resource, Default)]
struct BroadcastSent(bool);

fn reader1(
    mut reader: MessageReader<broadcast_message::Announcement>,
    mut count: ResMut<Reader1Count>,
) {
    for _msg in reader.read() {
        count.0 += 1;
    }
}

fn reader2(
    mut reader: MessageReader<broadcast_message::Announcement>,
    mut count: ResMut<Reader2Count>,
) {
    for _msg in reader.read() {
        count.0 += 1;
    }
}

fn send_broadcast(
    mut writer: MessageWriter<broadcast_message::Announcement>,
    mut sent: ResMut<BroadcastSent>,
) {
    if sent.0 {
        return;
    }
    sent.0 = true;
    writer.write(broadcast_message::Announcement(
        "Hello everyone!".to_string(),
    ));
}

#[test]
fn test_multiple_readers_same_message() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Reader1Count>();
    app.init_resource::<Reader2Count>();
    app.init_resource::<BroadcastSent>();

    app.add_message::<broadcast_message::Announcement>();

    // Both readers should be able to read the same message
    app.add_systems(
        Update,
        (
            send_broadcast,
            reader1.after(send_broadcast),
            reader2.after(send_broadcast),
        ),
    );

    app.update();
    app.update();

    let count1 = app.world().resource::<Reader1Count>();
    let count2 = app.world().resource::<Reader2Count>();

    assert_eq!(count1.0, 1, "Reader 1 should have read 1 message");
    assert_eq!(count2.0, 1, "Reader 2 should have read 1 message");
}
