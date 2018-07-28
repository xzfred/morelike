
use twox_hash::{XxHash};


const BUFSIZE: usize = 1024;

// type Output<N> = GenericArray<u8, N>;
// static DEFAULT_SUM: [u8; 20] = [0; 20];

type HashSum = u64;

pub fn checksum(path: &PathBuf, all: bool) -> Result<HashSum> {
    let file: File = File::open(path)?;

    let mut buf: [u8; BUFSIZE] = [0; BUFSIZE];

    let mut handle = file.take(BUFSIZE as u64);
    let mut hash = XxHash::with_seed(0);
    loop {
        let read_size = handle.read(&mut buf)?;
        if read_size > 0 {
            hash.write(&buf.as_ref());
            if !all {
                break;
            }
        } else {
            break;
        }
    }
    let sum = hash.finish();
    Ok(sum)
}
