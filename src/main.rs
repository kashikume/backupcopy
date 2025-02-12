mod backupcopy;
use backupcopy::fsscanner::FsScanner;

fn main() {
    let source_path = String::from("C:\\Users\\akola\\source");
    let dest_path = String::from("C:\\Users\\akola\\test_copy");
    let mut source = FsScanner::scan(&source_path).unwrap();
    let mut destination = FsScanner::scan(&dest_path).unwrap();
    println!("source: {:?}", source);
    println!("destination: {:?}", destination);
}
