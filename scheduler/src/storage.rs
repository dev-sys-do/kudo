use std::collections::HashMap;

/// Defining a trait called IStorage that takes a generic type T.
trait IStorage<T> {
    fn get(&self, id: &str) -> Option<&T>;
    fn get_mut(&mut self, id: &str) -> Option<&mut T>;
    fn update(&mut self, id: &str, value: T);
    fn delete(&mut self, id: &str);
    fn get_all(&self) -> &HashMap<String, T>;
}

/// `Storage` is a generic type that stores a `HashMap` of `String` keys and `T` values.
/// 
/// Properties:
/// 
/// * `data`: This is the HashMap that will store the data.
#[derive(Debug, Default)]
pub struct Storage<T> {
    data: HashMap<String, T>,
}

impl<T> Storage<T> {
    /// `new` creates a new `Storage` instance.
    pub fn new() -> Self {
        Storage {
            data: HashMap::new(),
        }
    }
}

impl<T> IStorage<T> for Storage<T> {
    /// Get returns a reference to the value associated with the given key, or None if the key is not
    /// present in the map.
    /// 
    /// Arguments:
    /// 
    /// * `id`: The id of the object to get.
    /// 
    /// Returns:
    /// 
    /// A reference to the value in the hashmap.
    fn get(&self, id: &str) -> Option<&T> {
        self.data.get(id)
    }

    /// Get a mutable reference to the value associated with the given key.
    /// 
    /// Arguments:
    /// 
    /// * `id`: The id of the object to get.
    /// 
    /// Returns:
    /// 
    /// A mutable reference to the value in the map.
    fn get_mut(&mut self, id: &str) -> Option<&mut T> {
        self.data.get_mut(id)
    }
    
    /// `update` takes a mutable reference to a `HashMap` and a `String` and returns nothing
    /// 
    /// Arguments:
    /// 
    /// * `id`: The id of the data to update.
    /// * `value`: The value to be stored in the cache.
    fn update(&mut self, id: &str, value: T) {
        self.data.insert(id.to_string(), value);
    }
    
    /// Remove the element with the given id from the data map.
    /// 
    /// Arguments:
    /// 
    /// * `id`: The id of the data to delete.
    fn delete(&mut self, id: &str) {
        self.data.remove(id);
    }

    /// It returns a reference to the HashMap.
    /// 
    /// Returns:
    /// 
    /// A reference to the HashMap
    fn get_all(&self) -> &HashMap<String, T> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_insert() {
        let mut storage = Storage::new();
        storage.update("test", "test");
        assert_eq!(storage.get("test"), Some(&"test"));
    }

    #[test]
    fn test_storage_get() {
        let mut storage = Storage::new();
        storage.update("test", "test");
        assert_eq!(storage.get("test"), Some(&"test"));
    }

    #[test]
    fn test_storage_get_mut() {
        let mut storage = Storage::new();
        storage.update("test", "test");
        assert_eq!(storage.get_mut("test"), Some(&mut "test"));
    }

    #[test]
    fn test_storage_delete() {
        let mut storage = Storage::new();
        storage.update("test", "test");
        storage.delete("test");
        assert_eq!(storage.get("test"), None);
    }    
}
