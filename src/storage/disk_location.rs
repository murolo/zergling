use storage;
use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::path::Path;
use storage::{NeedleMapType, ReplicaPlacement, Result, Volume, VolumeId, TTL};


pub struct DiskLocation {
    pub directory: String,
    pub max_volume_count: i64,
    pub volumes: HashMap<VolumeId, Volume>,
}


impl DiskLocation {
    pub fn new(dir: &str, max_volume_count: i64) -> DiskLocation {
        DiskLocation {
            directory: String::from(dir),
            max_volume_count: max_volume_count,
            volumes: HashMap::new(),
        }
    }

    pub fn concurrent_loading_volumes(
        &mut self,
        _needle_map_kind: NeedleMapType,
        _concurrent: bool,
    ) {
        panic!("TODO");
    }


    // return (vid, collection)
    pub fn volume_id_from_path(&self, p: &Path) -> Result<(VolumeId, String)> {
        if p.is_dir() || p.extension().unwrap_or_default() != "dat" {
            return Err(box_err!("not valid file: {}", p.to_str().unwrap()));
        }

        let name = p.file_name().unwrap().to_str().unwrap();

        let collection: &str;
        let id: &str;
        if let Some(idx) = name.find("_") {
            collection = &name[0..idx];
            id = &name[idx + 1..name.len() - 4];
        } else {
            collection = &name[0..0];
            id = &name[0..name.len() - 4];
        }

        let vid = id.parse()?;

        Ok((vid, collection.to_string()))
    }

    pub fn load_existing_volumes(&mut self, needle_map_kind: NeedleMapType) -> Result<()> {
        // TODO concurrent load volumes
        // self.concurrent_loading_volumes(needle_map_kind, true);
        let dir = Path::new(&self.directory);
        debug!("load_existing_volumes dir: {}", self.directory);
        for entry in fs::read_dir(dir)? {
            let file = entry?.path();
            let fpath = file.as_path();

            debug!("get file: {:?}", fpath);

            if fpath.extension().unwrap_or_default() == "dat" {
                debug!("load volume for dat file {:?}", fpath);
                match self.volume_id_from_path(fpath) {
                    Ok((vid, collection)) => {
                        if self.volumes.get(&vid).is_some() {
                            continue;
                        }
                        let vr = Volume::new(
                            &self.directory,
                            &collection,
                            vid,
                            needle_map_kind,
                            ReplicaPlacement::default(),
                            TTL::default(),
                            0,
                        );
                        match vr {
                            Ok(v) => {
                                info!("add volume: {}", vid);
                                self.volumes.insert(vid, v);
                            }
                            Err(err) => {
                                error!("load volume {} err: {}", err, vid);
                            }
                        }
                    }
                    Err(err) => {
                        error!("{}", err);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn delete_volume(&mut self, vid: VolumeId) -> Result<()> {
        if let Some(v) = self.volumes.get_mut(&vid) {
            v.destroy()?;
        }

        self.volumes.remove(&vid);

        Ok(())
    }
}
