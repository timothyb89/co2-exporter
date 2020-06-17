use hidapi::{HidApi, HidDevice, HidError};
use snafu::{Snafu, ResultExt, ErrorCompat};
use lazy_static::lazy_static;

const VENDOR_ID: u16 = 0x04d9;
const PRODUCT_ID: u16 = 0xa052;
const MAGIC_WORD: &str = "Htemp99e";

const CODE_END: u8 = 0x0D;
const CODE_CO2: u8 = 0x50;
const CODE_TEMPERATURE: u8 = 0x42;

#[derive(Debug, Snafu)]
enum Error {
  #[snafu(display(
    "HID API error: {}", source
  ))]
  HidApiError {
    source: HidError
  },

  #[snafu(display(
    "Unable to open USB device"
  ))]
  DeviceOpenError,

  #[snafu(display(
    "Checksum error"
  ))]
  ChecksumError
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
enum Measurement {
  /// Measured temperature in degrees Celsius
  Temperature(f32),

  /// Measured CO2 concentration in PPM
  CO2(u32)
}

impl Measurement {
  fn from_raw_temperature(value: u32) -> Measurement {
    Measurement::Temperature((value as f32) * 0.0625 - 273.15)
  }
}

fn list_to_longint(bytes: &[u8; 8]) -> u64 {
  bytes.iter()
    .rev()
    .enumerate()
    .map(|(i, b)| ((*b as u64) << (i * 8)))
    .sum::<u64>()
}

fn longint_to_list(x: u64) -> [u8; 8] {
  const BYTES: [u8; 8] = [56, 48, 40, 32, 24, 16, 8, 0];

  let mut buf: [u8; 8] = [0; 8];
  for (i, b) in BYTES.iter().enumerate() {
    buf[i] = ((x >> b) & 0xFF) as u8
  }

  buf
}

fn gen_magic_word() -> [u8; 8] {
  let mut ret: [u8; 8] = [0; 8];
  for (i, byte) in MAGIC_WORD.as_bytes().iter().enumerate() {
    ret[i] = ((byte << 4) & (0xFF as u8)) | (byte >> 4);
  }

  ret
}

fn decrypt(bytes: &[u8; 8]) -> [u8; 8] {
  lazy_static! {
    static ref MAGIC_WORD_BYTES: [u8; 8] = gen_magic_word();
  }

  const SHUFFLE: [usize; 8] = [2, 4, 0, 7, 1, 6, 5, 3];

  let mut unshuffled: [u8; 8] = [0; 8];
  for (i_src, i_dest) in SHUFFLE.iter().enumerate() {
    unshuffled[*i_dest] = bytes[i_src];
  }

  let msg = list_to_longint(&unshuffled);
  
  // this is just 0?
  let magic_table_int: u64 = 0;

  let res = msg ^ magic_table_int;
  let res = (res >> 3) | ((res << 61) & 0xFFFFFFFFFFFFFFFF);

  let res_list = longint_to_list(res);

  // iterators can only collect into a vec...
  let mut decrypted: [u8; 8] = [0; 8];
  for i in 0..8 {
    let res_byte = res_list[i];
    let magic_byte = MAGIC_WORD_BYTES[i];

    decrypted[i] = ((res_byte as i16 - magic_byte as i16) & 0xFF) as u8;
  }

  decrypted
}

fn verify_checksum(bytes: &[u8]) -> bool {
  eprintln!("bytes: {:?}", bytes);

  if bytes[5] != 0 || bytes[6] != 0 || bytes[7] != 0 {
    return false;
  }

  if bytes[4] != CODE_END {
    return false;
  }

  // lsb of sum of first 3 bytes
  let sum = bytes.iter().take(3).map(|b| *b as u32).sum::<u32>();
  if (sum & 0xff) as u8 != bytes[3] {
    return false;
  }

  true
}

fn read_once(device: &HidDevice) -> Result<Option<Measurement>> {
  let mut buf: [u8; 8] = [0; 8];
  device.read(&mut buf).context(HidApiError)?;

  let decrypted = decrypt(&buf);
  if !verify_checksum(&decrypted) {
    return Ok(None)
  }

  let op = decrypted[0];
  let value = (decrypted[1] as u32) << 8 | (decrypted[2] as u32);

  let ret = match op {
    CODE_CO2 => Some(Measurement::CO2(value)),
    CODE_TEMPERATURE => Some(Measurement::from_raw_temperature(value)),
    _ => None
  };

  Ok(ret)
}

fn run() -> Result<()> {
  let api = HidApi::new().context(HidApiError)?;
  let device = api.open(VENDOR_ID, PRODUCT_ID).context(HidApiError)?;

  println!(
    "device: manufacturer={}, product={}, serial={}",
    device.get_manufacturer_string().context(HidApiError)?.unwrap_or("n/a".into()),
    device.get_product_string().context(HidApiError)?.unwrap_or("n/a".into()),
    device.get_serial_number_string().context(HidApiError)?.unwrap_or("n/a".into())
  );

  device.send_feature_report(&[0, 0, 0, 0, 0, 0, 0, 0]).context(HidApiError)?;

  for _ in 0..100 {
    println!("{:?}", read_once(&device)?);
    std::thread::sleep(std::time::Duration::from_millis(1000));
  }

  Ok(())
}

fn main() {
  match run() {
    Ok(()) => (std::process::exit(0)),
    Err(e) => {
      eprintln!("An error occurred: {}", e);
      if let Some(backtrace) = ErrorCompat::backtrace(&e) {
        eprintln!("{}", backtrace);
      }

      std::process::exit(1);
    }
  }
}
