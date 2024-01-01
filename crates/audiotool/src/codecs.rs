use rx::prelude::*;
use std::path::Path;
use crate::io::*;
use crate::types::*;
use std::ffi::OsStr;

pub fn reader(path: &Path) -> AnyResult<Box<dyn PcmReader>> {
    let ext = path.extension()
        .map(OsStr::to_string_lossy)
        .as_deref()
        .map(str::to_string);
    match ext.as_deref() {
        Some("wav") => {
            Ok(Box::new(wav::WavPcmReader::new(path)))
        }
        Some("flac") => {
            Ok(Box::new(flac::FlacPcmReader::new(path)))
        }
        Some("ogg") => {
            Ok(Box::new(vorbis::VorbisPcmReader::new(path)))
        }
        Some(ext) => {
            Err(anyhow!("unknown extension: `{ext}`"))
        }
        None => {
            Err(anyhow!("no file extension"))
        }
    }
}

pub fn writer(
    path: &Path,
    props: Props,
) -> Box<dyn PcmWriter> {
    match props.format.codec {
        Codec::Wav => {
            Box::new(wav::WavPcmWriter::new(path, props))
        }
        Codec::Flac => {
            Box::new(flac::FlacPcmWriter::new(path, props))
        }
        Codec::Vorbis => {
            Box::new(vorbis::VorbisPcmWriter::new(path, props))
        }
    }
}

pub mod wav {
    use rx::prelude::*;
    use crate::types::{Format, BitDepth, SampleRate, Codec};
    use crate::io::{PcmReader, PcmWriter, Buf, Props};
    use std::path::Path;
    use std::io::{BufReader, BufWriter};
    use std::fs::File;

    pub struct WavPcmReader {
        reader: hound::Result<hound::WavReader<BufReader<File>>>,
    }

    impl WavPcmReader {
        pub fn new(path: &Path) -> WavPcmReader {
            WavPcmReader {
                reader: hound::WavReader::open(path),
            }
        }
    }

    impl PcmReader for WavPcmReader {
        fn props(&mut self) -> AnyResult<Props> {
            let reader = self.reader.as_ref()
                .map_err(|e| anyhow!("{e}"))?;
            let spec = reader.spec();
            Ok(Props {
                channels: spec.channels,
                format: Format {
                    codec: Codec::Wav,
                    bit_depth: match (spec.bits_per_sample, spec.sample_format) {
                        (32, hound::SampleFormat::Float) => BitDepth::F32,
                        (24, hound::SampleFormat::Int) => BitDepth::I24,
                        (16, hound::SampleFormat::Int) => BitDepth::I16,
                        (bits, format) => bail!("unsupported sample format: {bits}/{format:?}"),
                    },
                    sample_rate: match spec.sample_rate {
                        48_000 => SampleRate::K48,
                        192_000 => SampleRate::K192,
                        r => bail!("unsupported sample rate: {r} hz"),
                    }
                }
            })
        }

        fn read(
            &mut self,
            buf: &mut Buf,
        ) -> AnyResult<()> {
            let props = self.props()?;
            let reader = self.reader.as_mut()
                .map_err(|e| anyhow!("{e}"))?;
            match props.format.bit_depth {
                BitDepth::F32 => {
                    let bytes_to_read = 4096 * props.channels as usize;
                    let mut buf = buf.f32_mut();
                    buf.truncate(0);
                    buf.reserve_exact(bytes_to_read);
                    let mut samples = reader.samples::<f32>();
                    for _ in 0..bytes_to_read {
                        match samples.next() {
                            Some(sample) => {
                                buf.push(sample?);
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Ok(())
                }
                BitDepth::I24 => {
                    let bytes_to_read = 4096 * props.channels as usize;
                    let mut buf = buf.i24_mut();
                    buf.truncate(0);
                    buf.reserve_exact(bytes_to_read);
                    let mut samples = reader.samples::<i32>();
                    for _ in 0..bytes_to_read {
                        match samples.next() {
                            Some(sample) => {
                                buf.push(sample?);
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Ok(())
                }
                BitDepth::I16 => {
                    let bytes_to_read = 4096 * props.channels as usize;
                    let mut buf = buf.i16_mut();
                    buf.truncate(0);
                    buf.reserve_exact(bytes_to_read);
                    let mut samples = reader.samples::<i16>();
                    for _ in 0..bytes_to_read {
                        match samples.next() {
                            Some(sample) => {
                                buf.push(sample?);
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Ok(())
                }
            }            
        }
    }

    pub struct WavPcmWriter {
        writer: Option<hound::Result<hound::WavWriter<BufWriter<File>>>>,
    }

    impl WavPcmWriter {
        pub fn new(
            path: &Path,
            props: Props,
        ) -> WavPcmWriter {
            assert_eq!(props.format.codec, Codec::Wav);
            let spec = hound::WavSpec {
                channels: props.channels,
                sample_rate: props.format.sample_rate.as_u32(),
                bits_per_sample: match props.format.bit_depth {
                    BitDepth::F32 => 32,
                    BitDepth::I24 => 24,
                    BitDepth::I16 => 16,
                },
                sample_format: match props.format.bit_depth {
                    BitDepth::F32 => hound::SampleFormat::Float,
                    BitDepth::I24 => hound::SampleFormat::Int,
                    BitDepth::I16 => hound::SampleFormat::Int,
                },
            };
            WavPcmWriter {
                writer: Some(hound::WavWriter::create(path, spec)),
            }
        }
    }

    impl PcmWriter for WavPcmWriter {
        fn write(
            &mut self,
            buf: &Buf,
        ) -> AnyResult<()> {
            match &mut self.writer {
                Some(writer) => {
                    let writer = writer.as_mut()
                        .map_err(|e| anyhow!("{e}"))?;
                    match buf {
                        Buf::F32(buf) => {
                            for sample in buf.iter().copied() {
                                writer.write_sample(sample)?;
                            }
                        }
                        Buf::I24(buf) => {
                            for sample in buf.iter().copied() {
                                writer.write_sample(sample)?;
                            }
                        }
                        Buf::I16(buf) => {
                            for sample in buf.iter().copied() {
                                writer.write_sample(sample)?;
                            }
                        }
                        Buf::Uninit => panic!(),
                    }
                    Ok(())
                }
                None => {
                    panic!("already finalized");
                }
            }
        }

        fn finalize(&mut self) -> AnyResult<()> {
            let writer = std::mem::replace(&mut self.writer, None);
            match writer {
                Some(writer) => {
                    let writer = writer
                        .map_err(|e| anyhow!("{e}"))?;
                    writer.finalize()?;
                    Ok(())
                }
                None => {
                    panic!("already finalized");
                }
            }
        }
    }
}

pub mod flac {
    use rx::prelude::*;
    use crate::types::{Format, BitDepth, SampleRate, Codec};
    use crate::io::{PcmReader, PcmWriter, Buf, Props};
    use std::path::Path;
    use std::io::{BufReader, BufWriter};
    use std::fs::File;
    use std::ptr::NonNull;
    use std::ffi::{c_void, CStr};
    use libflac_sys::*;
    use rx::libc::c_char;

    pub struct FlacPcmReader {
        decoder: AnyResult<NonNull<FLAC__StreamDecoder>>,
        cbdata: *mut ReaderCallbackData,
    }

    struct ReaderCallbackData {
        props: Option<Props>,
        buf: Buf,
        error: AnyResult<()>,
    }

    unsafe impl Send for FlacPcmReader { }

    impl FlacPcmReader {
        pub fn new(path: &Path) -> FlacPcmReader {
            let mut cbdata = Box::new(ReaderCallbackData {
                props: None,
                buf: Buf::Uninit,
                error: Ok(()),
            });

            unsafe {
                let decoder = FLAC__stream_decoder_new();
                let decoder = NonNull::new(decoder);
                let decoder = decoder.ok_or_else(|| {
                    anyhow!("unable to allocate FLAC decoder")
                });

                let decoder = if let Ok(decoder) = decoder {
                    FLAC__stream_decoder_set_md5_checking(decoder.as_ptr(), true as FLAC__bool);

                    use std::ffi::CString;
                    let path = path.to_str().expect("todo utf8 path").to_owned();
                    let path = CString::new(path).expect("path with nul bytes").to_owned();

                    let status = FLAC__stream_decoder_init_file(
                        decoder.as_ptr(),
                        path.as_ptr(),
                        Some(decoder_write_callback),
                        Some(decoder_metadata_callback),
                        Some(decoder_error_callback),
                        cbdata.as_mut() as *mut ReaderCallbackData as *mut c_void,
                    );

                    if status == FLAC__STREAM_DECODER_INIT_STATUS_OK {
                        Ok(decoder)
                    } else {
                        FLAC__stream_decoder_delete(decoder.as_ptr());
                        let err_str = code_to_string(&FLAC__StreamDecoderInitStatusString, status);
                        Err(anyhow!("{err_str}"))
                    }
                } else {
                    decoder
                };

                FlacPcmReader {
                    decoder,
                    cbdata: Box::leak(cbdata) as *mut ReaderCallbackData,
                }
            }
        }
    }

    unsafe fn code_to_string(
        table: &[*const c_char; 0],
        code: u32,
    ) -> String {
        let cstr_ptr = table.as_ptr().offset(code as isize);
        let cstr = CStr::from_ptr(*cstr_ptr);
        cstr.to_str().expect("utf8").to_owned()
    }

    extern "C" fn decoder_write_callback(
        decoder: *const FLAC__StreamDecoder,
        frame: *const FLAC__Frame,
        buffer: *const *const i32,
        cbdata: *mut c_void,
    ) -> FLAC__StreamDecoderWriteStatus {
        assert!(!decoder.is_null());
        assert!(!frame.is_null());
        assert!(!buffer.is_null());
        assert!(!cbdata.is_null());

        unsafe {
            let cbdata = &mut *(cbdata as *mut ReaderCallbackData);
            todo!()
        }
    }

    extern "C" fn decoder_metadata_callback(
        decoder: *const FLAC__StreamDecoder,
        metadata: *const FLAC__StreamMetadata,
        cbdata: *mut c_void,
    ) {
        assert!(!decoder.is_null());
        assert!(!metadata.is_null());
        assert!(!cbdata.is_null());

        unsafe {
            let cbdata = &mut *(cbdata as *mut ReaderCallbackData);

            if (*metadata).type_ == FLAC__METADATA_TYPE_STREAMINFO {
                let stream_info = &(*metadata).data.stream_info;

                let bit_depth = match stream_info.bits_per_sample {
                    24 => BitDepth::I24,
                    16 => BitDepth::I16,
                    _ => todo!("flac bits per sample"),
                };

                let sample_rate = match stream_info.sample_rate {
                    192_000 => SampleRate::K192,
                    48_000 => SampleRate::K48,
                    _ => todo!("flac sample rate"),
                };

                let channels = match stream_info.channels {
                    1 => 1,
                    2 => 2,
                    _ => todo!("flac channels"),
                };

                let props = Props {
                    channels,
                    format: Format {
                        codec: Codec::Flac,
                        bit_depth,
                        sample_rate,
                    }
                };

                assert!(cbdata.props.is_none());

                cbdata.props = Some(props);
            }
        }
    }

    extern "C" fn decoder_error_callback(
        decoder: *const FLAC__StreamDecoder,
        status: FLAC__StreamDecoderErrorStatus,
        cbdata: *mut c_void,
    ) {
        assert!(!decoder.is_null());
        assert!(!cbdata.is_null());

        unsafe {
            let cbdata = &mut *(cbdata as *mut ReaderCallbackData);

            let err_str = code_to_string(&FLAC__StreamDecoderErrorStatusString, status);
            cbdata.error = Err(anyhow!("{err_str}"));
        }
    }

    impl Drop for FlacPcmReader {
        fn drop(&mut self) {
            unsafe {
                if let Ok(decoder) = self.decoder.as_ref() {
                    FLAC__stream_decoder_delete(decoder.as_ptr());
                }

                let _cbdata = Box::from_raw(self.cbdata);
            }
        }
    }

    impl PcmReader for FlacPcmReader {
        fn props(&mut self) -> AnyResult<Props> {
            let decoder = self.decoder.as_ref()
                .map_err(|e| anyhow!("{e}"))?;

            unsafe {
                // Take and drop references to the shared cbdata
                // before calling the decoder, which will mutate them.
                {
                    let error = &(*self.cbdata).error;

                    if let Err(e) = error {
                        bail!("{e}");
                    }

                    let props = &(*self.cbdata).props;

                    if let Some(props) = props {
                        return Ok(*props);
                    }
                };

                let ok = FLAC__stream_decoder_process_until_end_of_metadata(decoder.as_ptr());

                if ok != 0 {
                    assert!((*self.cbdata).props.is_some());
                    self.props()
                } else {
                    let state = FLAC__stream_decoder_get_state(decoder.as_ptr());
                    let err_str = code_to_string(&FLAC__StreamDecoderStateString, state);
                    Err(anyhow!("{err_str}"))
                }
            }
        }

        fn read(
            &mut self,
            buf: &mut Buf,
        ) -> AnyResult<()> {
            todo!()
        }
    }

    pub struct FlacPcmWriter {
    }

    impl FlacPcmWriter {
        pub fn new(
            path: &Path,
            props: Props,
        ) -> FlacPcmWriter {
            assert_eq!(props.format.codec, Codec::Flac);
            todo!()
        }
    }

    impl PcmWriter for FlacPcmWriter {
        fn write(
            &mut self,
            buf: &Buf,
        ) -> AnyResult<()> {
            todo!()
        }

        fn finalize(&mut self) -> AnyResult<()> {
            todo!()
        }
    }
}

pub mod vorbis {
    use rx::prelude::*;
    use crate::types::{Format, BitDepth, SampleRate, Codec};
    use crate::io::{PcmReader, PcmWriter, Buf, Props};
    use std::path::Path;
    use std::io::{BufReader, BufWriter};
    use std::fs::File;

    pub struct VorbisPcmReader {
    }

    impl VorbisPcmReader {
        pub fn new(path: &Path) -> VorbisPcmReader {
            VorbisPcmReader {
            }
        }
    }

    impl PcmReader for VorbisPcmReader {
        fn props(&mut self) -> AnyResult<Props> {
            todo!()
        }

        fn read(
            &mut self,
            buf: &mut Buf,
        ) -> AnyResult<()> {
            todo!()
        }
    }

    pub struct VorbisPcmWriter {
    }

    impl VorbisPcmWriter {
        pub fn new(
            path: &Path,
            props: Props,
        ) -> VorbisPcmWriter {
            assert_eq!(props.format.codec, Codec::Vorbis);
            todo!()
        }
    }

    impl PcmWriter for VorbisPcmWriter {
        fn write(
            &mut self,
            buf: &Buf,
        ) -> AnyResult<()> {
            todo!()
        }

        fn finalize(&mut self) -> AnyResult<()> {
            todo!()
        }
    }
}
