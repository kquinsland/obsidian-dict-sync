use std::collections::HashSet;
use std::convert::AsRef;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;

use std::io::Write;
use std::path::PathBuf;

const CHECKSUM_PRELUDE: &str = "checksum_v1 = ";

#[derive(Debug)]
/// Represents a checksummed list of words used by Electron based apps for custom spell checking.
pub struct UserDictionary {
    // Custom dictionary is just a text file; one word per line
    pub path: Option<PathBuf>,
    pub words: Option<HashSet<String>>,

    // And the last line of the file is a checksum of all the words
    // TODO: is there a way to make this public but also read only?
    pub hash: Option<md5::Digest>,
}

impl AsRef<UserDictionary> for UserDictionary {
    fn as_ref(&self) -> &UserDictionary {
        self
    }
}

impl fmt::Display for UserDictionary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "UserDictionary {{ path: {:?}, word count: {:?}, hash: {:?} }}",
            self.path.as_ref().unwrap().to_str().unwrap(),
            // Dumping possibly hundreds of words to the log is not helpful; can use debug! for that if needed
            self.words.as_ref().unwrap().len(),
            format!("{:?}", self.hash.unwrap())
        )
    }
}

impl UserDictionary {
    /// Create a new UserDictionary from an existing file on disk
    /// Return Error if the file path can't be fully canonicalized
    pub fn new_from_file_path(dict_file: &str) -> Result<Self, io::Error> {
        // User given string goes in fully expanded PathBuf comes out
        let dict_file_path = get_hydrated_path_from_str(dict_file)?;
        // And then resolve to fqdn path. This will fail if the file does not exist.

        let dict_file_path = dict_file_path.as_path().canonicalize()?;
        // get_words_from_file will raise Error if the file can't be found but
        // we _just_ checked that with canonicalize()? so we don't need to handle the same
        // error condition ... twice, back to back.
        let filtered_words = get_words_from_file(&dict_file_path)?;
        // Return UserDictionary with the hashed words
        Ok(UserDictionary {
            path: Some(dict_file_path),
            hash: Some(calculate_hash_digest(&filtered_words)),
            words: Some(filtered_words),
        })
    }

    pub fn new_from_pathbuf(dict_file: PathBuf) -> Result<Self, io::Error> {
        let dict_file_path = get_hydrated_path_from_pathbuf(&dict_file)?;

        let dict_file_path = dict_file_path.as_path().canonicalize()?;
        // get_words_from_file will raise Error if the file can't be found but
        // we _just_ checked that with canonicalize()? so we don't need to handle the same
        // error condition ... twice, back to back.
        let filtered_words = get_words_from_file(&dict_file_path)?;
        // Return UserDictionary with the hashed words
        Ok(UserDictionary {
            path: Some(dict_file_path),
            hash: Some(calculate_hash_digest(&filtered_words)),
            words: Some(filtered_words),
        })
    }

    /// Adds words to the dictionary and recalculates the hash
    pub fn add_words(&mut self, words: HashSet<String>) {
        // TODO: don't forget to remove filter words
        match &self.words {
            None => {
                self.words = Some(words);
            }
            Some(_) => {
                self.words = Some(
                    self.words
                        .as_ref()
                        .unwrap()
                        .union(&words)
                        .cloned()
                        .collect(),
                );
            }
        }
        // In either case, update the hash
        self.hash = Some(calculate_hash_digest(self.words.as_ref().unwrap()));
    }

    /// Removes words from the dictionary and recalculates the hash
    pub fn remove_words(&mut self, words: HashSet<String>) {
        debug!(
            "Attempting removal of '{}' words from dictionary.",
            words.len()
        );
        // TODO: don't forget to remove filter words
        match &self.words {
            None => {
                warn!("No words to remove from dictionary!");
                return;
            }
            Some(_) => {
                self.words = Some(
                    self.words
                        .as_ref()
                        .unwrap()
                        .difference(&words)
                        .cloned()
                        .collect(),
                );
            }
        }
        // In either case, update the hash
        self.hash = Some(calculate_hash_digest(self.words.as_ref().unwrap()));
    }

    /// Set the words in the dictionary and recalculate the hash
    pub fn set_words(&mut self, words: HashSet<String>) {
        self.words = Some(words);
        self.hash = Some(calculate_hash_digest(self.words.as_ref().unwrap()));
    }

    /// Get the unique words,, sorted
    fn get_sorted_words(&self) -> Vec<&String> {
        let mut words = self.words.as_ref().unwrap().iter().collect::<Vec<_>>();
        words.sort();
        words
    }

    /// Writes the dictionary words and hash to disk
    /// Creates the file if it doesn't exist.
    /// Overwrites the file if it does exist.
    /// Returns an error if the file can't be written to.
    pub fn write_to_disk(&mut self) -> Result<(), io::Error> {
        // Get cannon file path, create if it doesn't exist
        let canonical_path_result = canonicalize_or_create(self.path.as_ref().unwrap()).ok();
        debug!("Writing dictionary to '{:?}'", canonical_path_result);

        let dict_file_path = self.path.as_ref().unwrap().canonicalize()?;
        debug!(
            "Writing dictionary to '{}'",
            dict_file_path.to_str().unwrap()
        );

        // Internally, we're using the words as a HashSet for uniqueness
        let words = self.get_sorted_words();
        debug!("Writing '{:#?}' words to disk...", words.len());
        // Add the checksum line to the end of the file
        // TODO: implement Display for the hash?
        let checksum_line = format!("{}{:#?}", CHECKSUM_PRELUDE, self.hash.as_ref().unwrap());

        // Write the words to the file
        let mut file = File::create(&dict_file_path)?;
        for word in words {
            file.write_all(word.as_bytes())?;
            file.write_all(b"\n")?;
        }
        // Checksum line is last line, should not have a newline
        file.write_all(checksum_line.as_bytes())?;
        // TODO: moar/better error handling
        Ok(())
    }
}

fn canonicalize_or_create(path: &PathBuf) -> io::Result<PathBuf> {
    match fs::canonicalize(path) {
        Ok(canonical_path) => Ok(canonical_path),
        Err(e) => {
            // Check if the error is because the file does not exist
            if e.kind() == io::ErrorKind::NotFound {
                // Attempt to create the file
                File::create(path)?;
                // Retry canonicalization after creating the file
                fs::canonicalize(path)
            } else {
                error!("Error canonicalizing path: {:?}", e);
                Err(e)
            }
        }
    }
}

/// Returns all unique words in a given file
fn get_words_from_file(dict_file: &PathBuf) -> Result<HashSet<String>, io::Error> {
    if !dict_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{}' not found!", dict_file.to_str().unwrap()),
        ));
    }

    //TODO: file type is text / contains words?
    debug!(
        "Reading dictionary file '{}'...",
        dict_file.to_str().unwrap()
    );

    // Let caller deal with file not found
    let content = fs::read_to_string(dict_file)?;

    // Assuming we read the file, we care about every line other than the hash line.
    // A valid dictionary will have the file hash on the last line but we want to tolerate
    // dictionary files that might have the hash line anywhere.
    let dictionary_words: Vec<String> = content
        .lines()
        .map(String::from)
        .filter(|line| !line.contains(CHECKSUM_PRELUDE))
        .collect();

    debug!(
        "After filtering '{}', have a total of {} words...",
        dict_file.to_str().unwrap(),
        dictionary_words.len()
    );

    Ok(HashSet::from_iter(dictionary_words))
}

pub fn get_hydrated_path_from_str(dict_file: &str) -> Result<PathBuf, io::Error> {
    debug!("Resolving '{}'...", dict_file);

    // TODO: don't swallow errors around missing env-vars or whatever else causes full() to fail
    // See: https://docs.rs/shellexpand/latest/shellexpand/fn.full_with_context.html
    let dict_file = shellexpand::full(dict_file).unwrap().to_string();

    debug!("Resolved {:?}", dict_file);
    Ok(PathBuf::from(dict_file))
}

// Rather than overload, I think the way to do this is to implement a "hydrate" trait for PathBuf and str?
pub fn get_hydrated_path_from_pathbuf(dict_file: &PathBuf) -> Result<PathBuf, io::Error> {
    debug!("Resolving '{:?}'...", dict_file);

    // Convert PathBuf to string for shellexpand
    let dict_file_str = dict_file.to_str().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to convert PathBuf to str",
    ))?;

    // TODO: don't swallow errors around missing env-vars or whatever else causes full() to fail
    // See: https://docs.rs/shellexpand/latest/shellexpand/fn.full_with_context.html
    let dict_file = shellexpand::full(dict_file_str).unwrap().to_string();

    debug!("Resolved {:?}", dict_file);
    Ok(PathBuf::from(dict_file))
}

fn calculate_hash_digest(words: &HashSet<String>) -> md5::Digest {
    // Python code to calc the hash properly:
    //     _words = "".join(sorted(words))
    //     checksum = md5(_words.encode()).hexdigest()

    debug!("Calculating hash of '{}' words...", words.len());
    // HashSet ensures unique. We still need to sort before we can calc
    ////
    // Odd issue: https://github.com/rust-lang/rust/issues/82910
    // Asking for help here: https://old.reddit.com/r/learnrust/comments/11zwfi9/join_on_vec_or_strings_violates_trait_bounds/
    // Not sure why str works here but String does not :(
    let mut words = words.iter().map(String::as_str).collect::<Vec<_>>();

    // This took a while to figure out, had to reference python implementation in debugger and compare to what's going on here
    // Basically the lines() call is dropping the \n which needs to be present when we do the hashing
    // After sorting, add an empty word to the list so the join() call will add a \n at the very end of the string
    // Once we do that, the md5 hash calc will yield the correct hash
    words.sort();
    words.push("");

    let all_words = words.join("\n");
    let digest = md5::compute(&all_words);

    // Digest will be numbers that we just need to stringify rather than encode
    // See: https://stackoverflow.com/a/68052602
    debug!(
        "From '{}' words, we get '{}' chars and a digest of '{}'",
        words.len(),
        all_words.len(),
        format!("{:?}", &digest),
    );

    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn test_canonicalize_existing_file() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Hello, world!")?;

        let canonicalized = canonicalize_or_create(&temp_file.path().to_path_buf())?;
        assert!(canonicalized.is_absolute());
        Ok(())
    }

    #[test]
    fn test_create_and_canonicalize_nonexistent_file() -> io::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("nonexistent_file.txt");

        assert!(!file_path.exists());

        let canonicalized = canonicalize_or_create(&file_path)?;
        assert!(canonicalized.is_absolute());
        assert!(Path::new(&canonicalized).exists());
        Ok(())
    }

    #[test]
    fn test_error_on_invalid_path() {
        // Attempting to create a file in a non-existent directory within a temporary directory
        // to simulate an invalid path scenario.
        let temp_dir = tempfile::tempdir().unwrap();
        let invalid_path = temp_dir
            .path()
            .join("non_existent_directory/test_invalid_file.txt");

        let result = canonicalize_or_create(&invalid_path);
        assert!(result.is_err());
        let error = result.err().unwrap();
        // The specific error kind might vary based on the operation and platform,
        // so checking for a generic error condition here might be more appropriate.
        assert!(
            error.kind() == io::ErrorKind::NotFound
                || error.kind() == io::ErrorKind::PermissionDenied
        );
    }
}
