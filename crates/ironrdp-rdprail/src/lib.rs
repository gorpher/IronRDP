mod pdu;

use tracing::debug;
use ironrdp_pdu::cursor::ReadCursor;
use ironrdp_pdu::gcc::ChannelName;
use ironrdp_pdu::{PduDecode, PduResult};
use ironrdp_pdu::rdp::server_license::Scope;
use ironrdp_svc::{impl_as_any, CompressionCondition, SvcClientProcessor, SvcMessage, SvcProcessor};
use crate::pdu::{RailDataPdu, RailExecOrder, RailHighContrast, RailSysParamOrder, Rectangle16};

/// We currently don't implement any of rdpsnd, however it's required
/// for rdpdr to work: [\[MS-RDPEFS\] Appendix A<1>]
///
/// [\[MS-RDPEFS\] Appendix A<1>]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdperp/d2c7f732-cbd7-483c-b409-0c7c308a9879
#[derive(Debug)]
pub struct Rdprail;

impl Rdprail {
    pub const NAME: ChannelName = ChannelName::from_static(b"RAIL\0\0\0\0");

    pub fn new() -> Self {
        Self
    }
}

impl Default for Rdprail {
    fn default() -> Self {
        Self::new()
    }
}

impl_as_any!(Rdprail);

impl Rdprail {
    fn send_client_status(&self) -> RailDataPdu {
        let mut flags: u32 = pdu::TS_RAIL_CLIENTSTATUS_ALLOWLOCALMOVESIZE;
        //clientStatus.flags |= TS_RAIL_CLIENTSTATUS_AUTORECONNECT;

        flags |= pdu::TS_RAIL_CLIENTSTATUS_ZORDER_SYNC;
        flags |= pdu::TS_RAIL_CLIENTSTATUS_WINDOW_RESIZE_MARGIN_SUPPORTED;
        flags |= pdu::TS_RAIL_CLIENTSTATUS_APPBAR_REMOTING_SUPPORTED;
        flags |= pdu::TS_RAIL_CLIENTSTATUS_POWER_DISPLAY_REQUEST_SUPPORTED;
        flags |= pdu::TS_RAIL_CLIENTSTATUS_BIDIRECTIONAL_CLOAK_SUPPORTED;
        RailDataPdu::CLIENTSTATUS {
            flags
        }
    }
    fn send_client_system_param(&self) -> Vec<RailDataPdu> {
        let mut vec: Vec<RailDataPdu> = Vec::new();
        vec.push(RailDataPdu::SYSPARAM(RailSysParamOrder::new_with_high_contrast(
            RailHighContrast {
            flags: 0x7E,
            colorSchemeLength: 0,
            colorScheme: "".to_string(),
        })));
        vec.push(RailDataPdu::SYSPARAM(RailSysParamOrder::new_with_mouse_button_swap(0)));
        vec.push(RailDataPdu::SYSPARAM(RailSysParamOrder::new_with_keyboard_pref(0)));
        vec.push(RailDataPdu::SYSPARAM(RailSysParamOrder::new_with_drag_full_windows(0)));
        vec.push(RailDataPdu::SYSPARAM(RailSysParamOrder::new_with_keyboard_cues(0)));
        vec.push(RailDataPdu::SYSPARAM(RailSysParamOrder::new_with_work_area(Rectangle16 {
            left: 0,
            top: 0,
            right: 1024, // TODO
            bottom: 768,
        })));
        vec
    }

    fn send_client_exec(&self) -> RailDataPdu {
        RailDataPdu::EXEC(RailExecOrder {
            flags: 0,
            remote_application_program: "C:/Windows/notepad.exe".to_string(),
            remote_application_working_dir: "".to_string(),
            remote_application_arguments: "".to_string(),
        })
    }
}
impl SvcProcessor for Rdprail {
    fn channel_name(&self) -> ChannelName {
        Self::NAME
    }
    fn start(&mut self) -> PduResult<Vec<SvcMessage>> {
        return Ok(Vec::new());
    }
    fn compression_condition(&self) -> CompressionCondition {
        CompressionCondition::WhenRdpDataIsCompressed
    }

    fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {

        let mut src = ReadCursor::new(payload);
        let pdu = RailDataPdu::decode(&mut src)?;
        debug!("Rail Received {:?}", pdu);

        match pdu {
            RailDataPdu::HANDSHAKE_EX { builder_number,rail_handshake_flags} => {
                debug!("RAIL HANDSHAKE_EX builder_number {:?}", builder_number);
                let mut vec: Vec<SvcMessage> = Vec::new();
                let status = self.send_client_status();

                vec.push(SvcMessage::from(status));

                let sysparms = self.send_client_system_param();
                for x in sysparms {
                    vec.push(SvcMessage::from(x));
                }
                let exe = self.send_client_exec();
                vec.push(SvcMessage::from(exe));
                Ok(vec)
            }
            RailDataPdu::HANDSHAKE { builder_number } => {
                debug!("RAIL HANDSHAKE builder_number {:?}", builder_number);
                let mut vec: Vec<SvcMessage> = Vec::new();
                let status = self.send_client_status();

                vec.push(SvcMessage::from(status));
                //
                // let sysparms = self.send_client_system_param();
                // for x in sysparms {
                //     vec.push(SvcMessage::from(x));
                // }
                let exe = self.send_client_exec();
                vec.push(SvcMessage::from(exe));
                Ok(vec)
            }
            RailDataPdu::SYSPARAM(s) => {
                debug!("Rail 系统参数 {:?}",s);
                Ok(Vec::new())
            }
            RailDataPdu::EXEC_RESULT(result) => {
                debug!("Rail 执行结果 {:?}",result);

                Ok(Vec::new())
            }
            _ => {
                debug!("Rail 未知命令 {:?}",pdu);
                Ok((Vec::new()))
            }
        }
    }
}

impl SvcClientProcessor for Rdprail {}
