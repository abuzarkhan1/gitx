use gix;
fn main() {
    let repo = gix::open(".").unwrap();
    for r in repo.references().unwrap().all().unwrap() {
        let mut r = r.unwrap();
        let target = r.peeled_id().unwrap();
    }
}
