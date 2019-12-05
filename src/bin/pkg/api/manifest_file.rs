use crate::pkg::APIResult;
use serde::de::DeserializeOwned;
use serde::export::PhantomData;
use serde::Serialize;
use std::fs::File;
use std::io::Write;

pub struct ManifestFile<T> {
    file_name: String,
    phantom: PhantomData<T>,
}

impl<T> ManifestFile<T> {
    pub fn new(file_name: &str) -> ManifestFile<T> {
        ManifestFile {
            file_name: file_name.into(),
            phantom: PhantomData,
        }
    }
}

impl<T> ManifestFile<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    pub fn load(&self) -> APIResult<T> {
        match File::open(&self.file_name) {
            Ok(file) => Ok(serde_yaml::from_reader(file)?),
            _ => Ok(T::default()),
        }
    }

    pub fn save(&self, value: T) -> APIResult<()> {
        let mut file = File::create(&self.file_name)?;
        serde_yaml::to_writer(&mut file, &value)?;
        write!(file, "\n")?;
        Ok(())
    }

    pub fn update<R, F: FnOnce(&mut T) -> APIResult<R>>(&self, f: F) -> APIResult<R> {
        let mut value = self.load()?;
        let result = f(&mut value)?;
        self.save(value)?;
        Ok(result)
    }
}
