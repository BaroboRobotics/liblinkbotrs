#![allow(non_snake_case)]

#[macro_use] extern crate lazy_static;
extern crate linkbot_core as lc;
#[macro_use] extern crate log;
extern crate websocket as ws;

use std::env;
use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::thread;
use std::sync::{Mutex};

use ws::{Message};
use ws::client::ClientBuilder;

mod linkbot;

pub use linkbot::{Linkbot};

lazy_static! {
    static ref DAEMON: Mutex<lc::DaemonProxy> = {
        let d = Mutex::new(lc::DaemonProxy::new());
        init_daemon(&d);
        d
    };
}

const DEFAULT_DAEMON_HOST: &'static str = "ws://127.0.0.1:42000";

fn init_daemon(daemon: &Mutex<lc::DaemonProxy>) {
    // Create the websocket connection
    let uri = match env::var("LINKBOT_DAEMON_HOSTPORT") {
        Ok(address) => address,
        _ => String::from(DEFAULT_DAEMON_HOST)
    };
    let client = ClientBuilder::new(uri.as_str())
        .unwrap()
        .connect_insecure()
        .unwrap();
    let (mut ws_rx, mut ws_tx) = client.split().unwrap();
    //let (daemon_write_tx, daemon_write_rx) = sync_channel::<Vec<u8>>(1);
    // Set the daemon proxy's write callback
    {
        info!("Locking daemon in init_daemon()...");
        match daemon.try_lock() {
            Ok(mut d) => {
                info!("Daemon locked in init_daemon()...");
                d.set_write_callback(move |data| {
                    let message = Message::binary(data);
                    ws_tx.send_message(&message).unwrap();
                });
                info!("Daemon unlocked in init_daemon()...");
            } 
            _ => {
                panic!("Could not lock daemon lock!");
            }
        }
        /*
        let mut d = DAEMON.lock().unwrap();
        d.set_write_callback(move |data| {
            let message = Message::binary(data);
            ws_tx.send_message(&message).unwrap();
        });
        */
    }

    // Start the read pump
    thread::spawn( move || {
        info!("Read pump waiting for message...");
        for message_option in ws_rx.incoming_messages() {
            info!("Read pump waiting for message...Received message.");
            if message_option.is_err() {
                warn!("WS connection closed.");
                return;
            }
            let message = message_option.unwrap();
            {
                info!("Locking daemon...");
                let mut d = DAEMON.lock().unwrap();
                match message {
                    ws::OwnedMessage::Binary(data) => d.deliver(&data).unwrap(),
                    _ => warn!("Unexpected websocket data received.")
                }
                info!("Unlocking daemon...");
            }
        }
    });
}

#[no_mangle]
pub extern fn linkbotFromSerialId(serial_id: *mut c_char) -> *mut Linkbot {
    let robot = unsafe {
        let cstring = CString::from_raw(serial_id);
        let robot = Linkbot::new(cstring.to_str().unwrap());
        mem::forget(cstring);
        robot
    };
    Box::into_raw( Box::new(robot) )
}

#[no_mangle]
pub extern fn linkbotDelete(linkbot: *mut Linkbot) {
    let robot = unsafe {
        Box::from_raw(linkbot)
    };
}

// GETTERS

#[no_mangle]
pub extern fn linkbotGetAccelerometer(linkbot: *mut Linkbot, 
                                      x: *mut f64,
                                      y: *mut f64,
                                      z: *mut f64) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    let mut rc = 0;
    if let Ok((_x, _y, _z)) = robot.get_accelerometer_data() {
        unsafe {
        *x = _x as f64;
        *y = _y as f64;
        *z = _z as f64;
        }
    } else {
        rc = -1;
    }
    Box::into_raw(robot);
    rc
}

#[no_mangle]
pub extern fn linkbotGetBatteryVoltage(linkbot: *mut Linkbot, voltage: *mut f64) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotGetFormFactor(linkbot: *mut Linkbot, form: *mut i32) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotGetJointAngles(linkbot: *mut Linkbot, 
                                    timestamp: *mut i32,
                                    angle1: *mut f64,
                                    angle2: *mut f64,
                                    angle3: *mut f64) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    let mut rc = 0;
    if let Ok((_angle1, _angle2, _angle3)) = robot.get_joint_angles() {
        unsafe {
        *angle1 = _angle1 as f64;
        *angle2 = _angle2 as f64;
        *angle3 = _angle3 as f64;
        }
    } else {
        rc = -1;
    }
    Box::into_raw(robot);
    rc
}

#[no_mangle]
pub extern fn linkbotGetJointSpeeds(linkbot: *mut Linkbot, 
                                    timestamp: *mut i32,
                                    angle1: *mut f64,
                                    angle2: *mut f64,
                                    angle3: *mut f64) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    let mut rc = 0;
    if let Ok((_angle1, _angle2, _angle3)) = robot.get_joint_angles() {
        unsafe {
        *angle1 = _angle1 as f64;
        *angle2 = _angle2 as f64;
        *angle3 = _angle3 as f64;
        }
    } else {
        rc = -1;
    }
    Box::into_raw(robot);
    rc
}

#[no_mangle]
pub extern fn linkbotGetJointStates(linkbot: *mut Linkbot, 
                                    timestamp: *mut i32,
                                    state1: *mut i32,
                                    state2: *mut i32,
                                    state3: *mut i32) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotGetLedColor(linkbot: *mut Linkbot, 
                                 red: *mut i32,
                                 green: *mut i32,
                                 blue: *mut i32) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    let mut rc = 0;
    if let Ok((r,g,b)) = robot.get_led_color() {
        unsafe {
        *red = r as i32;
        *green = g as i32;
        *blue = b as i32;
        }
    } else {
        rc = -1;
    }
    Box::into_raw(robot);
    rc
}

#[no_mangle]
pub extern fn linkbotGetVersionString(linkbot: *mut Linkbot, 
                                      version: *mut c_char,
                                      n: i32) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotGetSerialId(linkbot: *mut Linkbot, 
                                 serial_id: *mut c_char) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotGetJointSafetyThresholds(linkbot: *mut Linkbot, 
                                              thresh1: *mut i32,
                                              thresh2: *mut i32,
                                              thresh3: *mut i32) -> i32
{
    unimplemented!();
}

pub extern fn linkbotGetJointSafetyAngles(linkbot: *mut Linkbot, 
                                          angle1: *mut f64,
                                          angle2: *mut f64,
                                          angle3: *mut f64) -> i32
{
    unimplemented!();
}

// SETTERS
pub extern fn linkbotSetAlphaI(linkbot: *mut Linkbot, 
                               mask: i32,
                               alpha1: *mut f64,
                               alpha2: *mut f64,
                               alpha3: *mut f64) -> i32
{
    unimplemented!();
}

pub extern fn linkbotSetAlphaF(linkbot: *mut Linkbot, 
                               mask: i32,
                               alpha1: *mut f64,
                               alpha2: *mut f64,
                               alpha3: *mut f64) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotResetEncoderRevs(linkbot: *mut Linkbot) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.reset_encoders().unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotSetBuzzerFrequency(linkbot: *mut Linkbot,
                                        frequency: f32) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.set_buzzer_frequency(frequency).unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotSetJointSpeeds(linkbot: *mut Linkbot,
                                    mask: i32,
                                    speed1: f64,
                                    speed2: f64,
                                    speed3: f64) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.set_joint_speeds(mask as u32, 
                           speed1 as f32, 
                           speed2 as f32, 
                           speed3 as f32).unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotSetJointStates(linkbot: *mut Linkbot,
                                    mask: i32,
                                    state1: i32, coef1: f64,
                                    state2: i32, coef2: f64,
                                    state3: i32, coef3: f64) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotSetJointStatesTimed(linkbot: *mut Linkbot,
                                         mask: i32,
                                         state1: i32, coef1: f64, end1: i32,
                                         state2: i32, coef2: f64, end2: i32,
                                         state3: i32, coef3: f64, end3: i32) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotSetLedColor(linkbot: *mut Linkbot,
                                 red: i32,
                                 green: i32,
                                 blue: i32) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.set_led_color(red as u8, green as u8, blue as u8).unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotSetJointSafetyThresholds(linkbot: *mut Linkbot,
                                              mask: i32,
                                              t1: i32,
                                              t2: i32,
                                              t3: i32) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotSetJointSafetyAngles(linkbot: *mut Linkbot,
                                          mask: i32,
                                          t1: f64,
                                          t2: f64,
                                          t3: f64) -> i32
{
    unimplemented!();
}

// MOVEMENT

#[no_mangle]
pub extern fn linkbotMoveAccel(linkbot: *mut Linkbot, 
                               mask: i32, relativeMask: i32,
                               a1: f64, timeout1: f64, endstate1: i32,
                               a2: f64, timeout2: f64, endstate2: i32,
                               a3: f64, timeout3: f64, endstate3: i32) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotMoveSmooth(linkbot: *mut Linkbot, 
                                mask: i32, relativeMask: i32,
                                angle1: f64,
                                angle2: f64,
                                angle3: f64) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotMoveContinuous(linkbot: *mut Linkbot, 
                                    mask: i32,
                                    d1: f64,
                                    d2: f64,
                                    d3: f64) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotMoveWait(linkbot: *mut Linkbot) -> i32 {
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.move_wait(0x07).unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotDrive(linkbot: *mut Linkbot, 
                           mask: i32,
                           d1: f64,
                           d2: f64,
                           d3: f64) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotDriveTo(linkbot: *mut Linkbot, 
                             mask: i32,
                             d1: f64,
                             d2: f64,
                             d3: f64) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotMotorPower(linkbot: *mut Linkbot, 
                                mask: i32,
                                d1: i32,
                                d2: i32,
                                d3: i32) -> i32
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotMove(linkbot: *mut Linkbot, 
                          mask: i32,
                          angle1: f64,
                          angle2: f64,
                          angle3: f64) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.move_motors(mask as u8, 
                      angle1 as f32, 
                      angle2 as f32, 
                      angle3 as f32).unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotMoveTo(linkbot: *mut Linkbot, 
                            mask: i32,
                            angle1: f64,
                            angle2: f64,
                            angle3: f64) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.move_motors_to(mask as u8, 
                         angle1 as f32, 
                         angle2 as f32, 
                         angle3 as f32).unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotStop(linkbot: *mut Linkbot, 
                          mask: i32) -> i32
{
    let mut robot = unsafe {
        Box::from_raw(linkbot)
    };
    robot.stop(mask as u32).unwrap();
    Box::into_raw(robot);
    0
}

#[no_mangle]
pub extern fn linkbotSetButtonEventCallback(linkbot: *mut Linkbot,
                                            cb: extern fn(i32, i32, i32, *mut c_void),
                                            user_data: *mut c_void)
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotSetEncoderEventCallback(linkbot: *mut Linkbot,
                                             cb: extern fn(i32, f64, i32, *mut c_void),
                                             user_data: *mut c_void)
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotSetJointEventCallback(linkbot: *mut Linkbot,
                                           cb: extern fn(i32, i32, i32, *mut c_void),
                                           user_data: *mut c_void)
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotSetAccelerometerEventCallback(linkbot: *mut Linkbot,
                                                   cb: extern fn(f64, f64, f64, i32, *mut c_void),
                                                   user_data: *mut c_void)
{
    unimplemented!();
}

#[no_mangle]
pub extern fn linkbotSetConnectionTerminatedCallback(linkbot: *mut Linkbot,
                                                     cb: extern fn(i32, *mut c_void),
                                                     user_data: *mut c_void)
{
    unimplemented!();
}

#[cfg(test)]
mod tests {
    extern crate colored_logger;
    use super::Linkbot;
    #[test]
    fn it_works() {
        colored_logger::init().unwrap();
        use std::io;
        let mut input = String::new();
        println!("Enter a robot's serial ID: ");
        if let Ok(n) = io::stdin().read_line(&mut input) {
            println!("Read serial-id, {} bytes: {}", n, input);
            input.truncate(4);
            let mut l = Linkbot::new(input.as_str());

            println!("Moving to zero...");
            l.reset_to_zero().unwrap();
            l.move_wait(0x07).unwrap();

            println!("Getting accelerometer data...");
            let acceldata = l.get_accelerometer_data().unwrap();
            println!("Accel data: {} {} {}", acceldata.0, acceldata.1, acceldata.2);
            
            println!("Setting LED color...");
            l.set_led_color(255, 255, 255).unwrap();

            println!("Setting robot button handler. Press 'Enter' to continue.");
            let button_handler = |timestamp, button, state| {
                println!("Button press! {}, {}, {}", timestamp, button, state);
            };
            l.enable_button_event(Some( Box::new( button_handler ) ) ).unwrap();
            io::stdin().read_line(&mut input);

            println!("Testing motors and buzzer. Moving motors 1, 2, and 3 90, 180, and 360 degrees, respectively...");
            l.set_buzzer_frequency(440.0);
            l.set_joint_speeds(0x07, 90.0, 90.0, 90.0).unwrap();
            l.move_motors(0x07, 90.0, 180.0, 360.0).unwrap();
            l.move_wait(0x07).unwrap();
            l.set_buzzer_frequency(0.0);
            println!("Test complete.");

            println!("Getting joint angles:");
            if let Ok(angles) = l.get_joint_angles() {
                println!("{} {} {}", angles.0, angles.1, angles.2);
            } else {
                panic!("Could not get joint angles.");
            }

            println!("Getting LED color:");
            if let Ok(color) = l.get_led_color() {
                println!("{} {} {}", color.0, color.1, color.2);
            } else {
                panic!("Could not get led color."); 
            }

            println!("Getting joint speeds:");
            if let Ok(speeds) = l.get_joint_speeds() {
                println!("{} {} {}", speeds.0, speeds.1, speeds.2);
            } else {
                panic!("Could not get joint speeds.");
            }

            println!("Setting joint speeds to 180...");
            l.set_joint_speeds(0x07, 180.0, 180.0, 180.0).unwrap();
            l.move_motors(0x07, 180.0, 180.0, 180.0).unwrap();
            l.move_wait(0x07).unwrap();
            l.set_joint_speeds(0x07, 90.0, 90.0, 90.0).unwrap();
            l.stop(0x07).unwrap();
        }
        
    }
}
