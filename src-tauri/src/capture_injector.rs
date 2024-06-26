
use std::{io::{ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, time::Duration};
use std::thread;

use tracing::{error, info};

use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};
use dll_syringe::{process::OwnedProcess, Syringe};
use chator_macros::sha256_to_array;

use crate::{share::CaptureMessage, swtor_hook};
use crate::dal::db::swtor_message::SwtorMessage;

pub mod message_container;
use self::message_container::SwtorMessageContainer;

const SUPPORTED_SWTOR_CHECKSUM: [u8; 32] = sha256_to_array!("9999679ECF122DF9B3E460B2C85E1FBE46F891E38841AF6A38CD79895F46D6D9");

lazy_static! {
    static ref INJECTED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref CONTINUE_LOGGING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref MESSAGE_CONTAINER: Arc<Mutex<SwtorMessageContainer>> = Arc::new(Mutex::new(SwtorMessageContainer::new()));
}

#[derive(Deserialize, Serialize)]
pub enum CaptureError {
    AlreadyInjected,
    SwtorNotRunning,
    WrongGuiSettings,
    UnsupportedVersion,
    NotYetFullyReady
}

#[tauri::command]
pub fn start_injecting_capture(window: tauri::Window) -> Result<(), CaptureError> {

    if INJECTED.load(Ordering::Relaxed) {
        return Err(CaptureError::AlreadyInjected);
    }

    let swtor_pid = swtor_hook::get_pid();
    if swtor_pid.is_none() {
        return Err(CaptureError::SwtorNotRunning);
    }
    let swtor_pid = swtor_pid.unwrap();

    match swtor_hook::checksum_match(&SUPPORTED_SWTOR_CHECKSUM) {
        Ok(true) => {},
        Ok(false) => return Err(CaptureError::UnsupportedVersion),
        Err(_) => return Err(CaptureError::NotYetFullyReady)
    }

    start_injecting_thread(swtor_pid, window);
    return Ok(());

}

fn start_injecting_thread(swtor_pid: u32, window: tauri::Window) {

    thread::spawn(move || {

        INJECTED.store(true, Ordering::Relaxed);
        CONTINUE_LOGGING.store(true, Ordering::Relaxed);
        
        let target_process = OwnedProcess::from_pid(swtor_pid).unwrap();
        let syringe = Syringe::for_process(target_process);

        let tcp_thread = thread::spawn(|| {
            start_tcp_listener_loop();
        });
        thread::sleep(Duration::from_secs(1));
        start_logging_propagation(window);

        let injected_payload = if cfg!(debug_assertions) {
            syringe.find_or_inject("./target/debug/swtor_chat_capture.dll")
        } else {
            syringe.find_or_inject("./swtor_chat_capture.dll")
        };

        match injected_payload {
            Ok(_) => {    
                info!("Payload injected");
            },
            Err(err) => {
                error!("Error injecting payload: {:?}", err);
                INJECTED.store(false, Ordering::Relaxed);
                CONTINUE_LOGGING.store(false, Ordering::Relaxed);
                return;
            }
        }

        let injected_payload = injected_payload.unwrap();

        tcp_thread.join().unwrap();

        if let Err(err) = syringe.eject(injected_payload) {
            error!("Error ejecting payload: {:?}", err);
        } else {
            info!("Payload ejected");
        }
        CONTINUE_LOGGING.store(false, Ordering::Relaxed);
        INJECTED.store(false, Ordering::Relaxed);

    });

}

fn start_tcp_listener_loop() {

    let listener = TcpListener::bind("127.0.0.1:4592").unwrap();
    let mut stream = listener.accept().unwrap().0;

    stream.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();

    info!("Listening for messages");
    let mut buffer: [u8; 2048] = [0; 2048];
    while CONTINUE_LOGGING.load(Ordering::Relaxed) {

        match stream.read(&mut buffer) {
            Ok(_) => {},
            Err(ref e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => {
                continue;
            },
            Err(err) => {
                error!("Error reading from stream: {:?}", err);
                break;
            }
        }

        Deserializer::from_slice(&buffer).into_iter::<Value>().for_each(|value| {

            if let Ok(value) = value {

                if let Ok(message) = serde_json::from_value(value) {

                    handle_message(message);

                }

            }

        });
        buffer = [0; 2048];
    }
    info!("Stopped listening for messages");

    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:4593") {
        stream.write(b"stop").unwrap();
    }
    thread::sleep(Duration::from_secs(1));

}

fn handle_message(message: CaptureMessage) {

    match message {

        CaptureMessage::Panic(panic_message) => {
            panic!("{}", panic_message);
        },
        _ => {
            MESSAGE_CONTAINER
                .lock()
                .unwrap()
                .push(message);
        }

    }

}

fn start_logging_propagation(window: tauri::Window) {

    let messages = Arc::clone(&MESSAGE_CONTAINER);
    thread::spawn(move || {

        while CONTINUE_LOGGING.load(Ordering::Relaxed) || !messages.lock().unwrap().unstored_messages.is_empty() {

            let unstored_messages = messages
                .lock()
                .unwrap()
                .drain_unstored();

            if !unstored_messages.is_empty() {
                SwtorMessage::save_messages_to_db(unstored_messages.clone());
                window.emit("swtor_messages", unstored_messages).unwrap();
            }

            thread::sleep(Duration::from_secs(1));

        }

    });

}

#[tauri::command]
pub fn stop_injecting_capture() {

    if !INJECTED.load(Ordering::Relaxed) {
        return;
    }

    CONTINUE_LOGGING.store(false, Ordering::Relaxed);
    
    while INJECTED.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
    }

}