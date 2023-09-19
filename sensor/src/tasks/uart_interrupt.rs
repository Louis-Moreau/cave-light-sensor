use chrono::NaiveDateTime;
use stm32l0xx_hal::prelude::_embedded_hal_serial_Read;
use stm32l0xx_hal::prelude::_embedded_hal_serial_Write;
use link_lib::*;
use nb::block;
use rtic::Mutex;
use stm32l0xx_hal::{
    exti::{DirectLine, Exti},
    serial::*,
};

use crate::{app::uart0, eeprom::{read_sensor_id, write_sensor_id, zero_stored_count, read_count_from_eeprom, read_event_from_eeprom}};

pub fn uart_interrupt(mut ctx: uart0::Context) {
    if Exti::is_pending(DirectLine::Lpuart1) {
        //let state = ctx.local.uart_int_state;
        let now = ctx.shared.rtc.lock(|r| r.now().timestamp() as u32);
        let delta_time = now - ctx.local.uart_int_state.last_timestamp;
        ctx.local.uart_int_state.last_timestamp = now;

        // Clear buffer if we haven't received a byte for more than 3s, this is needed because we
        // need to clear the buffer if we receive a corrupted message without a null byte at the end
        if delta_time > 3 {
            ctx.local.uart_int_state.message_buffer.clear_buffer();
        }

        while ctx.local.uart_int_state.uart_rx.is_rx_not_empty() {
            let byte = match block!(ctx.local.uart_int_state.uart_rx.read()) {
                Ok(b) => b,
                Err(_) => break,
            };
            if let Ok(Some(req)) = ctx
                .local
                .uart_int_state
                .message_buffer
                .add_byte_and_check_for_request(&byte)
            {
                let response = match req {
                    link_lib::Request::Ping => Response::Ok,
                    link_lib::Request::GetEmbeddedTime => {
                        Response::EmbeddedTime(ctx.shared.rtc.lock(|r| r.now().timestamp() as u32))
                    }
                    link_lib::Request::SetEmbeddedTime(t) => set_time(&mut ctx, t),
                    link_lib::Request::GetSensorId => get_sensor_id(&mut ctx),
                    link_lib::Request::SetSensorId(id) => set_sensor_id(&mut ctx,id),
                    link_lib::Request::ClearMemory => clear_memory(&mut ctx),
                    link_lib::Request::GetNumberofEvent => get_number_of_event(&mut ctx),
                    link_lib::Request::GetEvent(n) => get_event(&mut ctx,n),
                };
                ctx.local.uart_int_state.message_buffer.clear_buffer();
                let response_bytes = match Response::encode_message(&response) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                for byte in response_bytes {
                    match block!(ctx.local.uart_int_state.uart_tx.write(byte)) {
                        Ok(_) => (),
                        Err(_) => break,
                    };
                }

            }
        }
    }
}



fn set_time(ctx: &mut uart0::Context, timestamp: u32) -> Response {
    let new_time = match NaiveDateTime::from_timestamp_opt(timestamp as i64, 0) {
        Some(t) => t,
        None => return Response::Error,
    };

    return match ctx.shared.rtc.lock(|rtc| rtc.set(new_time)) {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    };
}

fn get_sensor_id(ctx: &mut uart0::Context) -> Response {
    match ctx.shared.eeprom.lock(|eeprom| read_sensor_id(eeprom)) {
        Ok(id) => Response::SensorId(id),
        Err(_) => Response::Error,
    }
}

fn set_sensor_id(ctx: &mut uart0::Context, sensor_id: u64) -> Response {
    match ctx.shared.eeprom.lock(|eeprom| write_sensor_id(eeprom, sensor_id)) {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    }
}

fn clear_memory(ctx: &mut uart0::Context) -> Response {
    match ctx.shared.eeprom.lock(|eeprom|zero_stored_count(eeprom)) {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    }
}

fn get_number_of_event(ctx: &mut uart0::Context) -> Response {
    match ctx.shared.eeprom.lock(|eeprom|read_count_from_eeprom(eeprom)) {
        Ok(_) => Response::Ok,
        Err(_) => Response::Error,
    }
}

fn get_event(ctx: &mut uart0::Context, number : u32) -> Response {
    match ctx.shared.eeprom.lock(|eeprom|read_event_from_eeprom(eeprom,number)) {
        Ok(e) => Response::EventInfo(e),
        Err(_) => Response::Error,
    }
}

pub struct UartInterruptState {
    pub uart_rx: Rx<LPUART1>,
    pub uart_tx: Tx<LPUART1>,
    pub message_buffer: MessageBuffer<{ link_lib::MAX_REQUEST_SIZE }>,
    pub last_timestamp: u32,
}
