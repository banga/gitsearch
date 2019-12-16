mod index;

use git2::Repository;
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

fn build_commit_index(index: &mut Index<String>, path: &str) -> Result<(), git2::Error> {
    let repo = Repository::open(path).unwrap();

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME | git2::Sort::REVERSE);
    for rev in revwalk {
        let rev = match rev {
            Ok(rev) => rev,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(rev) {
            Ok(commit) => commit,
            Err(_) => continue,
        };
        let message = match commit.message() {
            Some(message) => message,
            None => continue,
        };
        let summary = match commit.summary() {
            Some(summary) => summary,
            None => continue,
        };
        index.add(message, format!("{} {}", commit.id(), summary));
    }

    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);
    let path: String = args.next().unwrap();
    let query: Vec<String> = args.collect();

    let now = Instant::now();
    let mut index = Index::<String>::new();
    build_commit_index(&mut index, &path).expect("Error building index");
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
