use super::*;

impl Fat16Impl {
    pub fn new(inner: impl BlockDevice<Block512>) -> Self {
        let mut block = Block::default();
        let block_size = Block512::size();

        inner.read_block(0, &mut block).unwrap();
        let bpb = Fat16Bpb::new(block.as_ref()).unwrap();

        trace!("Loading Fat16 Volume: {:#?}", bpb);

        // HINT: FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
        let fat_start = bpb.reserved_sector_count() as usize;
        /* FIXME: get the size of root dir from bpb */
        let root_dir_size = ((bpb.root_entries_count() as usize * DirEntry::LEN) + block_size - 1) / block_size;
        /* FIXME: calculate the first root dir sector */
        let first_root_dir_sector = fat_start + (bpb.fat_count() as usize * bpb.sectors_per_fat() as usize);
        let first_data_sector = first_root_dir_sector + root_dir_size;

        Self {
            bpb,
            inner: Box::new(inner),
            fat_start,
            first_data_sector,
            first_root_dir_sector,
        }
    }

    pub fn cluster_to_sector(&self, cluster: &Cluster) -> usize {
        match *cluster {
            Cluster::ROOT_DIR => self.first_root_dir_sector,
            Cluster(c) => {
                // FIXME: calculate the first sector of the cluster
                let cluster_index = c - 2;
                // HINT: FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                cluster_index as usize * self.bpb.sectors_per_cluster() as usize + self.first_data_sector
            }
        }
    }

    // FIXME: YOU NEED TO IMPLEMENT THE FILE SYSTEM OPERATIONS HERE
    //      - calculate the sectors and the clusters
    //      - read the FAT and get cluster chain
    //      - traverse the cluster chain and read the data
    //      - parse the directory entries
    //      - ...
    //      - finally, implement the FileSystem trait for Fat16 with `self.handle`

    pub fn next_cluster(&self, cluster: &Cluster) -> Result<Cluster> {
        let fat_offset = (cluster.0 * 2) as usize;
        let mut block = Block::default();
        let size = Block512::size();
        let cur_fat_sector = self.fat_start + (fat_offset / size);
        let offset = fat_offset % size;
        self.inner.read_block(cur_fat_sector, &mut block).unwrap();
        let fat_entry = u16::from_le_bytes(block[offset..=offset + 1].try_into().unwrap_or([0; 2]));
        match fat_entry {
            0xFFF7 => Err(FsError::BadCluster),         // Bad cluster
            0xFFF8..=0xFFFF => Err(FsError::EndOfFile), // There is no next cluster
            f => Ok(Cluster(f as u32)),                 // Seems legit
        }
    }
    
    fn find_directory_entry(&self, dir: &Directory, name: &str) -> Result<DirEntry> {
        // info!("3");
        let match_name = ShortFileName::parse(name)?;
        let mut current_cluster = Some(dir.cluster);        
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        while let Some(cluster) = current_cluster {
            // info!("3");
            for sector in dir_sector_num..dir_sector_num + dir_size {
                match self.find_entry_in_sector(&match_name, sector) {
                    Err(FsError::NotInSector) => continue,
                    x => {
                        // info!("4");
                        // info!("{:#?}",x);
                        return x
                    },
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(&cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Err(FsError::FileNotFound)
    }

    fn find_entry_in_sector(&self, name: &ShortFileName, sector: usize) -> Result<DirEntry> {
        let mut block = Block::default();
        let size = Block512::size();
        self.inner.read_block(sector, &mut block).unwrap();
        // info!("{:#?}",block);
        for entry in 0..size / DirEntry::LEN {
            let start = entry * DirEntry::LEN;
            let end = (entry + 1) * DirEntry::LEN;
            let dir_entry = DirEntry::parse(&block[start..end]).map_err(|_| FsError::InvalidOperation)?;
            if dir_entry.is_eod() {
                info!("6");
                return Err(FsError::FileNotFound);
            } else if dir_entry.filename.matches(name) {
                return Ok(dir_entry);
            };
        }
        Err(FsError::NotInSector)
    }

    pub fn iterate_dir<F>(&self, dir: &directory::Directory, mut func: F) -> Result<()>
    where
        F: FnMut(&DirEntry),
    {
        if let Some(entry) = &dir.entry {
            trace!("Iterating directory: {}", entry.filename());
        }

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        trace!("Directory size: {}", dir_size);

        let mut block = Block::default();
        let block_size = Block512::size();
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                self.inner.read_block(sector, &mut block).unwrap();
                for entry in 0..block_size / DirEntry::LEN {
                    let start = entry * DirEntry::LEN;
                    let end = (entry + 1) * DirEntry::LEN;

                    let dir_entry = DirEntry::parse(&block[start..end])?;

                    if dir_entry.is_eod() {
                        return Ok(());
                    } else if dir_entry.is_valid() && !dir_entry.is_long_name() {
                        func(&dir_entry);
                    }
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(&cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Ok(())
    }

    fn get_parent_dir(&self, path: &str) -> Result<Directory> {
        // info!("2");
        let mut path = path.split(PATH_SEPARATOR);
        // trace!("get_parent_dir: {}", path.next().unwrap_or(""));
        let mut current = Directory::root();
        // trace!("current: {:?}", current);
        while let Some(dir) = path.next() {
            if dir.is_empty() {
                continue;
            }
            let entry = self.find_directory_entry(&current, dir)?;
            // trace!("entry: {:?}", entry);
            // info!("5");
            if entry.is_directory() {
                current = Directory::from_entry(entry);
            } else if path.next().is_some() {
                return Err(FsError::NotADirectory);
            } else {
                break;
            } 
        }
        // info!("2");
        Ok(current)
    }
    
    fn get_dir_entry(&self, path: &str) -> Result<DirEntry> {
        // info!("1");
        let parent = self.get_parent_dir(path)?;
        // info!("1");
        let name = path.rsplit(PATH_SEPARATOR).next().unwrap_or("");
        // info!("1");
        self.find_directory_entry(&parent, name)
    }
}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> Result<Box<dyn Iterator<Item = Metadata> + Send>> {
        // FIXME: read di>r and return an iterator for all entries
        // todo!()
        let dir = self.handle.get_parent_dir(path)?;
        // trace!("Iterating directory: {}", dir.entry.as_ref().unwrap().filename());
        let mut entries: Vec<Metadata> = Vec::new();
        self.handle.iterate_dir(&dir, |entry| {
            entries.push(entry.as_meta());
        })?;
        Ok(Box::new(entries.into_iter()))
    }

    fn open_file(&self, path: &str) -> Result<FileHandle> {
        // FIXME: open file and return a file handle
        // todo!()
        // trace!("open_file: {}", path);
        let entry = self.handle.get_dir_entry(path)?;
        // info!("111111111");
        // trace!("entry: {:?}", entry);
        if !entry.is_directory() {
            let handle = self.handle.clone();
            let meta = entry.as_meta();
            let file = Box::new(File::new(handle, entry));
            let file_handle = FileHandle::new(meta, file);
            Ok(file_handle)
        } else {
            Err(FsError::NotAFile)
        }
    }

    fn metadata(&self, path: &str) -> Result<Metadata> {
        // FIXME: read metadata of the file / dir
        // todo!()
        Ok(self.handle.get_dir_entry(path)?.as_meta())
    }

    fn exists(&self, path: &str) -> Result<bool> {
        // FIXME: check if the file / dir exists
        // todo!()
        Ok(self.handle.get_dir_entry(path).is_ok())
    }
}
