mod index;

use git2::{ObjectType, Repository};
use index::{Entry, Index};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;

fn build_file_index(index: &mut Index<String>, dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                build_file_index(index, &path)?;
            } else {
                let source = fs::read_to_string(entry.path()).unwrap();
                let path = String::from(entry.path().to_str().unwrap());
                index.add(&source, path);
            }
        }
    }
    Ok(())
}

fn build_commit_index(index: &mut Index<String>, path: &str) {
    let repo = Repository::open(path).unwrap();

    // TODO: Use revwalk or something else to only look at commits
    let mut count = 0;
    let odb = repo.odb().unwrap();
    odb.foreach(|oid| {
        let object = odb.read(*oid).unwrap();

        if object.kind() == ObjectType::Commit {
            let commit = repo.find_commit(*oid).unwrap();
            index.add(
                commit.message().unwrap(),
                format!("{}\n{}", commit.id(), commit.summary().unwrap()),
            );
            count += 1;
        }

        // Hack to stop the loop once it hits slow blob objects
        count < 150000
    })
    .unwrap();
}

fn main() {
    let mut args = env::args().skip(1);
    let path: String = args.next().unwrap();
    let query: Vec<String> = args.collect();

    let now = Instant::now();
    let mut index = Index::<String>::new();
    build_commit_index(&mut index, &path);
    let build_time = now.elapsed().as_millis();

    let now = Instant::now();
    let mut matches = Vec::<Entry>::new();
    index.find(&query.join(" "), &mut matches);
    let search_time = now.elapsed().as_millis();

    for (doc_id, _) in &matches {
        println!("{}", index.documents[*doc_id]);
    }

    println!("Index built in {}ms", build_time);
    println!("Search completed in {}ms", search_time);
    println!("{} results found", matches.len());
}
