//! Build-209026 health frames, IDs 256–260.
//! See `docs/re/209026-character-health-endurance.md` for lifetime requirements.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Five floats shared by creation and the server-259 value update.
/// The second meter's gameplay name and final value's meaning remain open.
#[derive(Debug, Clone, Copy)]
pub struct HealthValues209026 {
    pub current: f32,
    pub maximum: f32,
    pub second_current: f32,
    pub second_maximum: f32,
    pub extra: f32,
}

/// Wire operations only; no local game-state mutation or defaults.
#[derive(Debug, Clone, Copy)]
pub enum HealthOperation209026 {
    /// 256 creates or updates health; state applies only on the creation branch.
    CreateOrUpdate {
        /// Known labels: 0 alive, 1 dead, 2 downed. Other bytes are preserved.
        state: u8,
        values: HealthValues209026,
        /// First-meter units per second.
        rate: f32,
    },
    /// 257 requires existing health and changes both current values.
    Currents { current: f32, second_current: f32 },
    /// 258 removes existing health; it does not set a death state.
    Remove,
    /// 259 requires existing health and leaves its first-meter rate unchanged.
    Values(HealthValues209026),
    /// 260 requires existing health; first-meter units per second.
    Rate(f32),
}

#[derive(Debug, Clone, Copy)]
pub struct HealthFrame209026 {
    pub character_index: u32,
    pub operation: HealthOperation209026,
}

impl HealthFrame209026 {
    /// Append one complete frame; floats are retained without semantic checks.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use HealthOperation209026::*;
        let id: u16 = match self.operation {
            CreateOrUpdate { .. } => 256,
            Currents { .. } => 257,
            Remove => 258,
            Values(_) => 259,
            Rate(_) => 260,
        };
        id.encode(output);
        self.character_index.encode(output);
        match self.operation {
            CreateOrUpdate {
                state,
                values,
                rate,
            } => {
                state.encode(output);
                [
                    values.current,
                    values.maximum,
                    rate,
                    values.second_current,
                    values.second_maximum,
                    values.extra,
                ]
                .encode(output);
            }
            Currents {
                current,
                second_current,
            } => [current, second_current].encode(output),
            Remove => {}
            Values(values) => [
                values.current,
                values.maximum,
                values.second_current,
                values.second_maximum,
                values.extra,
            ]
            .encode(output),
            Rate(rate) => rate.encode(output),
        }
    }

    /// Consume one frame, leaving input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use HealthOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !(256..=260).contains(&id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected health message 256 through 260",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let operation = match id {
            256 => {
                let state = u8::decode(&mut cursor)?;
                let [
                    current,
                    maximum,
                    rate,
                    second_current,
                    second_maximum,
                    extra,
                ] = <[f32; 6]>::decode(&mut cursor)?;
                CreateOrUpdate {
                    state,
                    values: HealthValues209026 {
                        current,
                        maximum,
                        second_current,
                        second_maximum,
                        extra,
                    },
                    rate,
                }
            }
            257 => {
                let [current, second_current] = <[f32; 2]>::decode(&mut cursor)?;
                Currents {
                    current,
                    second_current,
                }
            }
            258 => Remove,
            259 => {
                let [current, maximum, second_current, second_maximum, extra] =
                    <[f32; 5]>::decode(&mut cursor)?;
                Values(HealthValues209026 {
                    current,
                    maximum,
                    second_current,
                    second_maximum,
                    extra,
                })
            }
            260 => Rate(f32::decode(&mut cursor)?),
            _ => unreachable!(),
        };
        *input = cursor;
        Ok(Self {
            character_index,
            operation,
        })
    }
}

/// Server 255 `{Dword character, Float delta, Float second_delta, Word
/// raw_word, Byte kind, Dword skill key, Dword source agent}` (handler
/// `0x14124f990`). Unlike 257 it adds deltas: `0x141275ee0` does
/// `current = clamp(current + delta, 0, maximum)` on health +0x8 and +0x24
/// (damage is negative; sandbox run 154 sent absolute values and the bars
/// stayed full). It then notifies the health listeners (list +0x48)
/// with the kind, the skill definition (content type 0x40; 0 = none) and the
/// source agent (resolved through `0x141024060`; 0 = none, a nonzero unknown
/// agent asserts). Wire kinds map to listener kinds as 0 -> 0, 1 -> 2,
/// 2 -> 3, 3 -> 1; others fail. Listeners receive the original deltas,
/// including when the current values were capped by their maxima.
/// Research codec: retail kind selection and raw_word semantics remain
/// unresolved; see docs/re/209026-health-gain-delivery.md.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourcedHealthFrame209026 {
    pub character_index: u32,
    pub delta: f32,
    pub second_delta: f32,
    pub raw_word: u16,
    pub kind: u8,
    pub skill_key: u32,
    pub source_agent: u32,
}

impl SourcedHealthFrame209026 {
    pub const MESSAGE_ID: u16 = 255;

    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.kind > 3 {
            return Err(Error::new(ErrorKind::InvalidInput, "health change kind must be 0..=3"));
        }
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        [self.delta, self.second_delta].encode(output);
        self.raw_word.encode(output);
        self.kind.encode(output);
        self.skill_key.encode(output);
        self.source_agent.encode(output);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sourced_health_255_matches_the_client_schema() {
        use crate::dynamic::{self, Rules};
        let frame = SourcedHealthFrame209026 {
            character_index: 3,
            delta: -30.0,
            second_delta: 0.0,
            raw_word: 0,
            kind: 0,
            skill_key: 10170,
            source_agent: 1,
        };
        let mut out = BytesMut::new();
        frame.encode_frame(&mut out).unwrap();
        let messages: Vec<_> = crate::schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml"))
            .unwrap()
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| g.name == "table-0x14216e940")
            .flat_map(|g| g.server)
            .collect();
        let mut input = &out[..];
        assert_eq!(dynamic::decode_frame(&messages, &Rules::default(), &mut input).unwrap().0.id, 255);
        assert!(input.is_empty());
        assert!(SourcedHealthFrame209026 { kind: 4, ..frame }.encode_frame(&mut BytesMut::new()).is_err());
    }

    fn fixtures() -> [Vec<u8>; 5] {
        [
            // 256, index 300, unknown state 255; six deliberately distinct floats.
            "00 01 ac 02 ff 00 00 80 3f 00 00 00 40 00 00 00 bf
             00 00 00 80 45 23 c1 7f 00 00 80 7f",
            "01 01 ac 02 00 00 00 80 45 23 c1 7f",
            "02 01 ac 02",
            "03 01 ac 02 00 00 80 3f 00 00 00 40 00 00 00 80 45 23 c1 7f 00 00 80 7f",
            "04 01 ac 02 00 00 00 bf",
        ]
        .map(|text| {
            text.split_whitespace()
                .map(|b| u8::from_str_radix(b, 16).unwrap())
                .collect()
        })
    }

    fn assert_values(v: HealthValues209026) {
        assert_eq!(
            [
                v.current,
                v.maximum,
                v.second_current,
                v.second_maximum,
                v.extra
            ]
            .map(f32::to_bits),
            [0x3f800000, 0x40000000, 0x80000000, 0x7fc12345, 0x7f800000]
        );
    }

    #[test]
    fn health_golden_variants_preserve_float_bits_and_state() {
        for (variant, golden) in fixtures().iter().enumerate() {
            let mut input = golden.as_slice();
            let frame = HealthFrame209026::decode_frame(&mut input).unwrap();
            assert_eq!(frame.character_index, 300);
            assert!(input.is_empty());
            match frame.operation {
                HealthOperation209026::CreateOrUpdate {
                    state,
                    values,
                    rate,
                } => {
                    assert_eq!(variant, 0);
                    assert_eq!(state, 255);
                    assert_values(values);
                    assert_eq!(rate, -0.5);
                }
                HealthOperation209026::Currents {
                    current,
                    second_current,
                } => {
                    assert_eq!(variant, 1);
                    assert_eq!(current.to_bits(), 0x80000000);
                    assert_eq!(second_current.to_bits(), 0x7fc12345);
                }
                HealthOperation209026::Remove => assert_eq!(variant, 2),
                HealthOperation209026::Values(values) => {
                    assert_eq!(variant, 3);
                    assert_values(values);
                }
                HealthOperation209026::Rate(rate) => {
                    assert_eq!(variant, 4);
                    assert_eq!(rate, -0.5);
                }
            }
            let mut encoded = BytesMut::from(&b"prefix"[..]);
            frame.encode_frame(&mut encoded);
            assert_eq!(&encoded[6..], golden);
        }
    }

    #[test]
    fn health_consecutive_frames_have_exact_boundaries() {
        let fixtures = fixtures();
        let stream = fixtures.concat();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for golden in &fixtures {
            let frame = HealthFrame209026::decode_frame(&mut input).unwrap();
            let mut encoded = BytesMut::new();
            frame.encode_frame(&mut encoded);
            assert_eq!(encoded, golden.as_slice());
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn health_bad_frames_leave_input_unchanged() {
        for golden in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(HealthFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        for malformed in [
            vec![0xff, 0, 1],
            vec![5, 1, 1],
            vec![2, 1, 0xff, 0xff, 0xff, 0xff, 0x10],
        ] {
            let mut input = malformed.as_slice();
            assert_eq!(
                HealthFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, malformed);
        }
    }
}
