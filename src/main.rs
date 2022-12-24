use windows::{
    core::*,
    Win32::*,
    Win32::System::*,
    Win32::System::Com::*,
    Win32::Foundation::*,
    Win32::Media::Audio::*,
    Win32::Media::Audio::Endpoints::*,
};

#[implement(IAudioEndpointVolumeCallback)]
struct MyVolumeControlCallback;

#[allow(non_snake_case)]
impl IAudioEndpointVolumeCallback_Impl for MyVolumeControlCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> ::windows::core::Result<()> {
        // let change = pnotify.read();
        let change = unsafe { pnotify.as_ref().unwrap() };

        println!("volume notification {} {}", change.fMasterVolume, change.nChannels);

        return Ok(());
    }
}

fn main() {
    println!("Hello, world!");

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).expect("TODO: panic message");

        let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_INPROC_SERVER
        )?;

        let device = device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        println!("device {}", device.GetId()?.0)
    }

    loop {}
}
