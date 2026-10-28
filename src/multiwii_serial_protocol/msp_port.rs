//#![allow(unused)]
use stream_buf::{StreamBufReader, StreamBufWriter};

use crate::{
    boards::{UsbReceiver, UsbSender},
    config::{ConfigPublisher, FastConfigPublisher},
};

use super::{
    Msp, MspSensorData, MspStream,
    msp::MspResult,
    msp_parser::{MspPacket, MspParser},
    msp_stream::MspPacketType,
};

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum MspPortState {
    #[default]
    Idle,
    Packet,
    CliActive,
    CliCmd,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum MspPendingSystemRequest {
    #[default]
    None,
    BootloaderRom,
    Cli,
    BootloaderFlash,
}

pub struct MspPort {
    //struct serialPort_s *port; // null when port unused.
    //mspPacketState_e packetState;
    parser: MspParser,
    last_activity_ms: u64,
    pending_request: MspPendingSystemRequest,
    port_state: MspPortState,
    packet_type: MspPacketType,
    //bool sharedWithTelemetry;
    //mspDescriptor_t descriptor;
    #[allow(unused)]
    pub usb_sender: UsbSender,
    #[allow(unused)]
    pub usb_receiver: UsbReceiver,
    //pub read_buf: [u8; Self::READ_BUF_SIZE],
    pub write_buf: [u8; Self::WRITE_BUF_SIZE],
}

impl MspPort {
    //pub const INBUF_SIZE: usize = 192;
    //const READ_BUF_SIZE: usize = 256;
    const WRITE_BUF_SIZE: usize = 512;

    pub const fn new(usb_sender: UsbSender, usb_receiver: UsbReceiver) -> Self {
        Self {
            parser: MspParser::new(),
            last_activity_ms: 0,
            pending_request: MspPendingSystemRequest::None,
            port_state: MspPortState::Idle,
            packet_type: MspPacketType::Command,
            usb_sender,
            usb_receiver,
            //read_buf: [0u8; Self::READ_BUF_SIZE],
            write_buf: [0u8; Self::WRITE_BUF_SIZE],
        }
    }
}

/*impl MspPort {
    /// Helper to get a reader for `read_buf`.
    pub fn stream_buf_reader(&'_ mut self) -> StreamBufReader<'_> {
        StreamBufReader::new(&self.read_buf)
    }

    /// Helper to get a writer for `write_buf`.
    pub fn stream_buf_writer(&'_ mut self) -> StreamBufWriter<'_> {
        StreamBufWriter::new(&mut self.write_buf)
    }
}*/

impl MspPort {
    fn serial_read(&mut self) -> u8 {
        _ = self;
        0u8
    }
    fn serial_rx_bytes_waiting(&mut self) -> bool {
        _ = self;
        false
        //ctx.usb_receiver.wait_connection().await;
    }

    pub async fn serial_process(
        &mut self,
        fast_config_publisher: &FastConfigPublisher,
        config_publisher: &ConfigPublisher,
        msp_sensor_data: &MspSensorData,
    ) {
        // whilst port is idle, poll incoming until portState changes or no more bytes
        while self.port_state == MspPortState::Idle && self.serial_rx_bytes_waiting() {
            // There are bytes incoming - abort pending request
            self.last_activity_ms = embassy_time::Instant::now().as_millis();
            self.pending_request = MspPendingSystemRequest::None;

            let byte = self.serial_read();
            if byte == b'$' {
                self.port_state = MspPortState::Packet;
                //mspPort->packetState = MSP_HEADER_START;
            }
        }

        match self.port_state {
            MspPortState::Idle => self.process_pending_request(),
            MspPortState::Packet => self.process_packet(fast_config_publisher, config_publisher, msp_sensor_data).await,
            _ => {}
        }
    }
    fn process_pending_request(&mut self) {
        let time_now_ms = embassy_time::Instant::now().as_millis();

        if self.pending_request == MspPendingSystemRequest::None || (time_now_ms - self.last_activity_ms < 100) {
            return;
        }

        #[allow(clippy::match_same_arms)]
        match self.pending_request {
            MspPendingSystemRequest::None => {}
            MspPendingSystemRequest::BootloaderRom => {}
            MspPendingSystemRequest::Cli => {}
            MspPendingSystemRequest::BootloaderFlash => {}
        }
    }
    async fn process_packet(
        &mut self,
        fast_config_publisher: &FastConfigPublisher,
        config_publisher: &ConfigPublisher,
        msp_sensor_data: &MspSensorData,
    ) {
        while self.serial_rx_bytes_waiting() {
            let byte = self.serial_read();

            //mspSerialProcessReceivedPacketData(mspPort, c);
            let option = self.parser.on_data_received(byte);

            if let Some(msp_packet) = option {
                if msp_packet.packet_type == MspPacketType::Command {
                    self.process_received_command(
                        &msp_packet,
                        fast_config_publisher,
                        config_publisher,
                        msp_sensor_data,
                    )
                    .await;
                } else if self.packet_type == MspPacketType::Reply {
                    self.process_received_reply();
                }
            }
        }

        /*if (mspPostProcessFn) {
            waitForSerialPortToFinishTransmitting(mspPort->port);
            mspPostProcessFn(mspPort->port);
        }*/
    }

    async fn process_received_command(
        &mut self,
        command_packet: &MspPacket,
        fast_config_publisher: &FastConfigPublisher,
        config_publisher: &ConfigPublisher,
        msp_sensor_data: &MspSensorData,
    ) {
        let mut dst = StreamBufWriter::new(&mut self.write_buf);

        #[allow(unused)]
        let mut msp_result =
            // write a config value into `dst`, eg Msp::BATTERY_CONFIG
            if Msp::process_write_command(command_packet.cmd, &mut dst, msp_sensor_data).await == MspResult::Ack {
                MspResult::Ack
            } else {
                let mut src = StreamBufReader::new(command_packet.payload());
                // read a config value from `src` and set it, eg Msp::SET_BATTERY_CONFIG
                Msp::process_read_command(command_packet.cmd, &mut src, config_publisher, fast_config_publisher).await
                //ret = mspCommonProcessInCommand(srcDesc, cmdMSP, src, mspPostProcessFn);
            };

        if msp_result != MspResult::NoReply {
            //sbufSwitchToReader(&reply.buf, outBufHead); // change streambuf direction
            //mspSerialEncode(msp, &reply, msp->mspVersion);
            let mut output_buf = [0u8; 256];
            let packet_type = MspPacketType::Reply;
            let payload = dst.get_data_slice();
            let result = MspStream::msp_encode(
                command_packet.msp_version,
                packet_type,
                command_packet.cmd,
                command_packet.flags,
                payload,
                &mut output_buf,
            );
            if let Ok(_size) = result {
                // TODO: write MSP encoded packet to serial.
            }
        }
        /*static uint8_t mspSerialOutBuf[MSP_PORT_OUTBUF_SIZE];

        mspPacket_t reply = {
            .buf = { .ptr = mspSerialOutBuf, .end = ARRAY END(mspSerialOutBuf), },
            .cmd = -1,
            .flags = 0,
            .result = 0,
            .direction = MSP_DIRECTION_REPLY,
        };
        uint8_t *outBufHead = reply.buf.ptr;

        mspPacket_t command = {
            .buf = { .ptr = msp->inBuf, .end = msp->inBuf + msp->dataSize, },
            .cmd = msp->cmdMSP,
            .flags = msp->cmdFlags,
            .result = 0,
            .direction = MSP_DIRECTION_REQUEST,
        };

        mspPostProcessFnPtr mspPostProcessFn = NULL;
        const mspResult_e status = mspFcProcessCommand(msp->descriptor, &command, &reply, &mspPostProcessFn);

        if (status != MSP_RESULT_NO_REPLY) {
            sbufSwitchToReader(&reply.buf, outBufHead); // change streambuf direction
            mspSerialEncode(msp, &reply, msp->mspVersion);
        }

        return mspPostProcessFn;*/
    }
    fn process_received_reply(&mut self) {
        _ = self;
        /*mspPacket_t reply = {
            .buf = {
                .ptr = msp->inBuf,
                .end = msp->inBuf + msp->dataSize,
            },
            .cmd = msp->cmdMSP,
            .result = 0,
        };

        mspFcProcessReply(&reply);*/
    }
}

/*
mspResult_e mspFcProcessCommand(mspDescriptor_t srcDesc, mspPacket_t *cmd, mspPacket_t *reply, mspPostProcessFnPtr *mspPostProcessFn)
{
    int ret = MSP_RESULT_ACK;
    sbuf_t *dst = &reply->buf;
    sbuf_t *src = &cmd->buf;
    const int16_t cmdMSP = cmd->cmd;
    // initialize reply by default
    reply->cmd = cmd->cmd;

    if (mspCommonProcessOutCommand(cmdMSP, dst, mspPostProcessFn)) {
        ret = MSP_RESULT_ACK;
    } else if (mspProcessOutCommand(srcDesc, cmdMSP, dst)) {
        ret = MSP_RESULT_ACK;
    } else if ((ret = mspFcProcessOutCommandWithArg(srcDesc, cmdMSP, src, dst, mspPostProcessFn)) != MSP_RESULT_CMD_UNKNOWN) {
        /* ret */;
    } else if (cmdMSP == MSP_SET_PASSTHROUGH) {
        mspFcSetPassthroughCommand(dst, src, mspPostProcessFn);
        ret = MSP_RESULT_ACK;
#ifdef USE_FLASHFS
    } else if (cmdMSP == MSP_DATAFLASH_READ) {
        mspFcDataFlashReadCommand(dst, src);
        ret = MSP_RESULT_ACK;
#endif
    } else {
        ret = mspCommonProcessInCommand(srcDesc, cmdMSP, src, mspPostProcessFn);
    }
    reply->result = ret;
    return ret;
}

void mspFcProcessReply(mspPacket_t *reply)
{
    sbuf_t *src = &reply->buf;
    UNUSED(src); // potentially unused depending on compile options.

    switch (reply->cmd) {
    case MSP_ANALOG:
        {
            uint8_t batteryVoltage = sbufReadU8(src);
            uint16_t mAhDrawn = sbufReadU16(src);
            uint16_t rssi = sbufReadU16(src);
            uint16_t amperage = sbufReadU16(src);

            UNUSED(rssi);
            UNUSED(batteryVoltage);
            UNUSED(amperage);
            UNUSED(mAhDrawn);

#ifdef USE_MSP_CURRENT_METER
            currentMeterMSPSet(amperage, mAhDrawn);
#endif
        }
        break;
    }
}*/

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<MspPort>();
        is_full::<MspPortState>();
        is_full::<MspPendingSystemRequest>();
    }
}
