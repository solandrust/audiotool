#![allow(unused)]

pub mod convert;
pub mod split;
pub mod io;
pub mod types;
pub mod codecs;

pub mod samplerate {
    use crate::types::SampleRate;
    use crate::io::Buf;

    pub struct SampleRateConverter {
        x: (),
    }

    impl SampleRateConverter {
        pub fn new(inrate: SampleRate, outrate: SampleRate) -> SampleRateConverter {
            todo!()
        }

        pub fn convert(&mut self, inbuf: &Buf) -> &Buf {
            todo!()
        }

        pub fn finalize(&mut self) -> &Buf {
            todo!()
        }
    }
}

pub mod bitdepth {
    use crate::types::BitDepth;
    use crate::io::Buf;

    pub struct BitDepthConverter {
        inbits: BitDepth,
        outbits: BitDepth,
        dither: bool,
        outbuf: Buf,
    }

    impl BitDepthConverter {
        pub fn new(inbits: BitDepth, outbits: BitDepth, origbits: BitDepth) -> BitDepthConverter {
            let dither = match (inbits, outbits, origbits) {
                (BitDepth::F32, BitDepth::F32, BitDepth::F32) => false,
                (BitDepth::F32, BitDepth::I24, BitDepth::F32) => false,
                (BitDepth::F32, BitDepth::I16, BitDepth::F32) => false,
                (BitDepth::F32, BitDepth::F32, BitDepth::I24) => false,
                (BitDepth::F32, BitDepth::I24, BitDepth::I24) => false,
                (BitDepth::F32, BitDepth::I16, BitDepth::I24) => true,
                (BitDepth::F32, BitDepth::F32, BitDepth::I16) => false,
                (BitDepth::F32, BitDepth::I24, BitDepth::I16) => false,
                (BitDepth::F32, BitDepth::I16, BitDepth::I16) => false,
                (_, _, _) => {
                    todo!()
                }
            };

            BitDepthConverter {
                inbits, outbits, dither,
                outbuf: Buf::Uninit,
            }
        }

        pub fn convert(&mut self, inbuf: &Buf) -> &Buf {
            match inbuf {
                Buf::Uninit => panic!(),
                Buf::F32(inbuf) => {
                    assert_eq!(self.inbits, BitDepth::F32);
                    match self.outbits {
                        BitDepth::F32 => {
                            assert!(!self.dither);
                            let mut outbuf = self.outbuf.f32_mut();
                            outbuf.truncate(0);
                            outbuf.extend(inbuf.iter());
                        }
                        _ => todo!()
                    }
                }
                _ => todo!(),
            }

            &self.outbuf
        }
    }
}

pub mod dither {
    pub fn i24(
        inbuf: &[i32],
        outbuf: &mut [i32],
    ) {
        todo!()
    }
}
