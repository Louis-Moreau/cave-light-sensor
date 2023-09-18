use chrono::NaiveDateTime;
use cortex_m::prelude::_embedded_hal_serial_Read;
use link_lib::{MessageBuffer, Response};
use nb::block;
use rtic::Mutex;
use stm32l0xx_hal::{
    exti::{DirectLine, Exti},
    serial::*,
};

use crate::{app::uart0, eeprom::read_sensor_id};

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
            let byte = block!(ctx.local.uart_int_state.uart_rx.read()).unwrap();
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
                    link_lib::Request::GetSensorId => {
                        Response::SensorId(ctx.shared.eeprom.lock(|eeprom| read_sensor_id(eeprom)))
                    }
                    link_lib::Request::SetSensorId(id) => todo!(),
                    link_lib::Request::ClearMemory => todo!(),
                    link_lib::Request::GetNumberofEvent => todo!(),
                    link_lib::Request::GetEvent(n) => todo!(),
                };
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

pub struct UartInterruptState {
    pub uart_rx: Rx<LPUART1>,
    pub uart_tx: Tx<LPUART1>,
    pub message_buffer: MessageBuffer<{ link_lib::MAX_REQUEST_SIZE }>,
    pub last_timestamp: u32,
}
