use std::fmt::Debug;
use std::thread::sleep;
use std::time::Duration;

#[allow(unused_imports)]
use windows::{
    core::*,
    Win32::*,
    Win32::UI::*,
    Win32::UI::Shell::PropertiesSystem::*,
    Win32::System::*,
    Win32::System::Com::*,
    Win32::Foundation::*,
    Win32::Media::Audio::*,
    Win32::Media::Audio::Endpoints::*,
};

#[implement(IAudioEndpointVolumeCallback)]
struct MyVolumeControlCallback {
    /*device: IMMDevice*/
}

#[allow(non_snake_case)]
impl IAudioEndpointVolumeCallback_Impl for MyVolumeControlCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> ::windows::core::Result<()> {
        // let change = pnotify.read();
        let change = unsafe { pnotify.as_ref().unwrap() };

        println!("volume notification {:#?} {} {} {} {}", change.guidEventContext, change.fMasterVolume, change.nChannels, change.bMuted.0, change.afChannelVolumes[0]);

        return Ok(());
    }
}

fn hook_audio_devices() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_INPROC_SERVER
        )?;

        //device_enumerator.RegisterEndpointNotificationCallback()

        let device = device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        // https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdevice-activate
        let audio_endpoint_volume = device.Activate::<IAudioEndpointVolume>(
            CLSCTX_ALL,
            None
        )?;

        // https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Media/Audio/Endpoints/struct.IAudioEndpointVolumeCallback.html
        let volume_control_cb = MyVolumeControlCallback {};
        let interface : IAudioEndpointVolumeCallback = volume_control_cb.into();
        audio_endpoint_volume.RegisterControlChangeNotify(&interface)?;

        loop {
            //println!("audio {}", audio_endpoint_volume.GetMasterVolumeLevelScalar()?);
            sleep(Duration::from_millis(10));
        }

        // https://rust-cli.github.io/book/in-depth/signals.html

        /*
        let device_coll = device_enumerator.EnumAudioEndpoints(
            eAll,
            DEVICE_STATE_ACTIVE | DEVICE_STATE_UNPLUGGED
        )?;

        for ndevice in 0..device_coll.GetCount()? {
            let device = device_coll.Item(ndevice)?;
            let property_store = device.OpenPropertyStore(STGM_READ)?;

            println!("ndevice {} {:?}: ", ndevice, device)

        }
        */
    }

    Ok(())
}

fn main() {
    println!("Hello, world!");

    hook_audio_devices().expect("Fail to hook audio devices");
}
