use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Word(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Spell(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Folder(u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Placed {
    pub file: Option<Spell>,
    pub folder: Option<Folder>,
}

struct Node {
    parent: Option<Folder>,
    word: Word,
    text: OnceLock<Arc<str>>,
}

const SEPARATORS: [char; 2] = ['\\', '/'];

#[derive(Default)]
pub struct Names {
    spellings: HashMap<Arc<str>, Spell>,
    spelled: Vec<(Arc<str>, Word)>,
    words: HashMap<Arc<str>, Word>,
    folded: Vec<Arc<str>>,
    nodes: HashMap<(Option<Folder>, Word), Folder>,
    folders: Vec<Node>,
    paths: HashMap<Arc<str>, Placed>,
}

impl Names {
    pub fn spell(&mut self, text: &str) -> Spell {
        if let Some(&spell) = self.spellings.get(text) {
            return spell;
        }
        let word = self.fold(text);
        let spell = Spell(self.spelled.len() as u32);
        let text: Arc<str> = Arc::from(text);
        self.spelled.push((text.clone(), word));
        self.spellings.insert(text, spell);
        spell
    }

    fn fold(&mut self, text: &str) -> Word {
        let folded = text.to_lowercase();
        if let Some(&word) = self.words.get(folded.as_str()) {
            return word;
        }
        let word = Word(self.folded.len() as u32);
        let folded: Arc<str> = Arc::from(folded);
        self.folded.push(folded.clone());
        self.words.insert(folded, word);
        word
    }

    pub fn word(&mut self, text: &str) -> Word {
        let spell = self.spell(text);
        self.word_of(spell)
    }

    pub fn word_of(&self, spell: Spell) -> Word {
        self.spelled[spell.0 as usize].1
    }

    pub fn text(&self, spell: Spell) -> &Arc<str> {
        &self.spelled[spell.0 as usize].0
    }

    pub fn folded(&self, word: Word) -> &Arc<str> {
        &self.folded[word.0 as usize]
    }

    pub fn find_word(&self, text: &str) -> Option<Word> {
        match self.spellings.get(text) {
            Some(&spell) => Some(self.word_of(spell)),
            None => self.words.get(text.to_lowercase().as_str()).copied(),
        }
    }

    pub fn find_folder(&self, path: &str) -> Option<Folder> {
        if path.is_empty() {
            return None;
        }
        let mut parent = None;
        for segment in path.split(SEPARATORS) {
            let word = self.find_word(segment)?;
            parent = Some(*self.nodes.get(&(parent, word))?);
        }
        parent
    }

    pub fn folder_text(&self, folder: Folder) -> Arc<str> {
        let node = &self.folders[folder.0 as usize];
        node.text
            .get_or_init(|| match node.parent {
                Some(parent) => format!(r"{}\{}", self.folder_text(parent), self.folded(node.word)).into(),
                None => self.folded(node.word).clone(),
            })
            .clone()
    }

    fn folder(&mut self, path: &str) -> Option<Folder> {
        if path.is_empty() {
            return None;
        }
        let mut parent = None;
        for segment in path.split(SEPARATORS) {
            let word = self.word(segment);
            let next = Folder(self.folders.len() as u32);
            let folder = *self.nodes.entry((parent, word)).or_insert(next);
            if folder == next {
                self.folders.push(Node {
                    parent,
                    word,
                    text: OnceLock::new(),
                });
            }
            parent = Some(folder);
        }
        parent
    }

    pub fn path(&mut self, path: &str) -> Placed {
        if let Some(&placed) = self.paths.get(path) {
            return placed;
        }
        let (folder, file) = path.rsplit_once(SEPARATORS).unwrap_or(("", path));
        let placed = Placed {
            file: (!file.is_empty()).then(|| self.spell(file)),
            folder: self.folder(folder),
        };
        self.paths.insert(Arc::from(path), placed);
        placed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_spelled_in_another_case_is_the_same_place() {
        let mut names = Names::default();
        let first = names.path(r"C:\Program Files\Git\git.exe");
        let second = names.path(r"c:\program files\GIT\Git.EXE");

        assert!(first.folder.is_some() && first.folder == second.folder, "{first:?} {second:?}");
        let words = (first.file.map(|file| names.word_of(file)), second.file.map(|file| names.word_of(file)));
        assert!(words.0.is_some() && words.0 == words.1, "{words:?}");
    }

    #[test]
    fn a_name_keeps_its_own_spelling_and_shares_its_word() {
        let mut names = Names::default();
        let lower = names.spell("git.exe");
        let upper = names.spell("Git.exe");

        assert_eq!((&**names.text(lower), &**names.text(upper)), ("git.exe", "Git.exe"));
        assert_eq!(names.word_of(lower), names.word_of(upper));
        assert_eq!(&**names.folded(names.word_of(upper)), "git.exe");
    }

    #[test]
    fn a_folder_reads_back_folded() {
        let mut names = Names::default();
        let placed = names.path(r"C:\Program Files\Git\cmd\git.exe");

        assert_eq!(
            placed.folder.map(|folder| names.folder_text(folder)).as_deref(),
            Some(r"c:\program files\git\cmd")
        );
    }

    #[test]
    fn a_folder_is_found_in_any_case_and_an_unknown_one_is_not() {
        let mut names = Names::default();
        let placed = names.path(r"C:\Tools\rg.exe");

        assert!(placed.folder.is_some() && names.find_folder(r"c:\TOOLS") == placed.folder, "{placed:?}");
        assert_eq!(names.find_folder(r"c:\tools\more"), None);
        assert_eq!(names.find_folder(r"d:\tools"), None);
        assert_eq!(names.find_word("RG.exe"), placed.file.map(|file| names.word_of(file)));
    }

    #[test]
    fn a_bare_file_has_no_folder() {
        let mut names = Names::default();
        let placed = names.path("git.exe");

        assert_eq!(placed.folder, None);
        assert_eq!(placed.file.map(|file| names.text(file).clone()).as_deref(), Some("git.exe"));
    }
}
