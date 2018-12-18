use std::fmt::{self, Debug};
use std::io::SeekFrom;
use std::path::Path;
use std::time::SystemTime;

use crate::error::{Error, Result};
use crate::fs::Config;
use crate::trans::eid::Eid;
use crate::util::crypto::Cipher;
use crate::util::crypto::Cost;
use crate::util::crypto::MemLimit;
use crate::util::crypto::OpsLimit;
use crate::util::time::Time;
use crate::util::version::Version;

#[derive(Debug, Default)]
pub struct RepoOpener {
    cfg: Config,
    create: bool,
    create_new: bool,
    read_only: bool,
}

impl RepoOpener {
    /// Creates a blank new set of options ready for configuration.
    #[inline]
    pub fn new() -> Self {
        RepoOpener::default()
    }

    /// Sets the password hash operation limit.
    ///
    /// This option is only used for creating a repository.
    /// `OpsLimit::Interactive` is the default.
    pub fn ops_limit(&mut self, ops_limit: OpsLimit) -> &mut Self {
        self.cfg.cost.ops_limit = ops_limit;
        self
    }

    /// Sets the password hash memory limit.
    ///
    /// This option is only used for creating a repository.
    /// `MemLimit::Interactive` is the default.
    pub fn mem_limit(&mut self, mem_limit: MemLimit) -> &mut Self {
        self.cfg.cost.mem_limit = mem_limit;
        self
    }

    /// Sets the crypto cipher encrypts the repository.
    ///
    /// This option is only used for creating a repository. `Cipher::Aes` is
    /// the default if hardware supports AES-NI instructions, otherwise it will
    /// fall back to `Cipher::Xchacha`.
    pub fn cipher(&mut self, cipher: Cipher) -> &mut Self {
        self.cfg.cipher = cipher;
        self
    }

    /// Sets the option for creating a new repository.
    ///
    /// This option indicates whether a new repository will be created if the
    /// repository does not yet already exist.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option to always create a new repository.
    ///
    /// This option indicates whether a new repository will be created. No
    /// repository is allowed to exist at the target location.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        if create_new {
            self.create = true;
        }
        self
    }

    /// Sets the option for data compression.
    ///
    /// This options indicates whether the LZ4 compression should be used in
    /// the repository. Default is false.
    pub fn compress(&mut self, compress: bool) -> &mut Self {
        self.cfg.compress = compress;
        self
    }

    /// Sets the default maximum number of file version.
    ///
    /// The `version_limit` must be within [1, 255], 10 is the default. This
    /// setting is a repository-wise setting, indivisual file can overwrite it
    /// by setting [`version_limit`] in [`OpenOptions`].
    ///
    /// [`version_limit`]: struct.OpenOptions.html#method.version_limit
    /// [`OpenOptions`]: struct.OpenOptions.html
    pub fn version_limit(&mut self, version_limit: u8) -> &mut Self {
        self.cfg.opts.version_limit = version_limit;
        self
    }

    /// Sets the default option for file data chunk deduplication.
    ///
    /// This option indicates whether data chunk should be deduped when
    /// writing data to a file. This setting is a repository-wise setting,
    /// indivisual file can overwrite it by setting [`dedup_chunk`]
    /// in [`OpenOptions`]. Default is true.
    ///
    /// [`dedup_chunk`]: struct.OpenOptions.html#method.dedup_chunk
    /// [`OpenOptions`]: struct.OpenOptions.html
    pub fn dedup_chunk(&mut self, dedup_chunk: bool) -> &mut Self {
        self.cfg.opts.dedup_chunk = dedup_chunk;
        self
    }

    /// Sets the option for read-only mode.
    ///
    /// This option cannot be true with either `create` or `create_new` is true.
    pub fn read_only(&mut self, read_only: bool) -> &mut Self {
        self.read_only = read_only;
        self
    }

    /// Opens a repository at URI with the password and options specified by
    /// `self`.
    ///
    /// Supported storage:
    ///
    /// - OS file system based storage, location prefix is `file://`
    ///
    ///   After the prefix is the path to a directory on OS file system. It can
    ///   be a relative or absolute path.
    ///
    /// - Memory based storage, location prefix is `mem://`
    ///
    ///   As memory stoage is volatile, it is always be used with `create`
    ///   option. It doesn't make sense to open an existing memory storage,
    ///   thus the string after prefix is arbitrary.
    ///
    /// - SQLite based storage, location prefix is `sqlite://`
    ///
    ///   After the prefix is the path to a SQLite database file. It can also
    ///   be a in-memory SQLite database, that is, the path can be ":memory:".
    ///   This storage can be enabled by feature `storage-sqlite`.
    ///
    /// - Redis based storage, location prefix is `redis://`
    ///
    ///   After the prefix is the path to a Redis instance. Unix socket is
    ///   supported. The URI format is:
    ///
    ///   `redis://[+unix+][:<passwd>@]<hostname>[:port][/<db>]`
    ///
    ///   This storage can be enabled by feature `storage-redis`.
    ///
    /// After a repository is opened, all of the other functions provided by
    /// ZboxFS will be thread-safe.
    ///
    /// The application should destroy the password as soon as possible after
    /// calling this function.
    ///
    /// # Errors
    ///
    /// Open a memory based repository without enable `create` option will
    /// return an error.
    pub fn open(&self, uri: &str, pwd: &str) -> Result<Repo> {
        // version limit must be greater than 0
        if self.cfg.opts.version_limit == 0 {
            return Err(Error::InvalidArgument);
        }

        if self.create {
            if self.read_only {
                return Err(Error::InvalidArgument);
            }
            if Repo::exists(uri)? {
                if self.create_new {
                    return Err(Error::AlreadyExists);
                }
                Repo::open(uri, pwd, self.read_only)
            } else {
                Repo::create(uri, pwd, &self.cfg)
            }
        } else {
            Repo::open(uri, pwd, self.read_only)
        }
    }
}

#[derive(Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    version_limit: Option<u8>,
    dedup_chunk: Option<bool>,
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to false, except for `read`.
    pub fn new() -> Self {
        OpenOptions {
            read: true,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            version_limit: None,
            dedup_chunk: None,
        }
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.write = write;
        self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous content. Note that setting
    /// `.write(true).append(true)` has the same effect as setting only
    /// `.append(true)`.
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.append = append;
        if append {
            self.write = true;
        }
        self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// Note that setting `.write(true).truncate(true)` has the same effect as
    /// setting only `.truncate(true)`.
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.truncate = truncate;
        if truncate {
            self.write = true;
        }
        self
    }

    /// Sets the option for creating a new file.
    ///
    /// This option indicates whether a new file will be created if the file
    /// does not yet already exist.
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.create = create;
        if create {
            self.write = true;
        }
        self
    }

    /// Sets the option to always create a new file.
    ///
    /// This option indicates whether a new file will be created. No file is
    /// allowed to exist at the target location.
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.create_new = create_new;
        if create_new {
            self.create = true;
            self.write = true;
        }
        self
    }

    /// Sets the maximum number of file versions allowed.
    ///
    /// The `version_limit` must be within [1, 255]. It will fall back to
    /// repository's [`version_limit`] if it is not set.
    ///
    /// [`version_limit`]: struct.RepoOpener.html#method.version_limit
    pub fn version_limit(&mut self, version_limit: u8) -> &mut OpenOptions {
        self.version_limit = Some(version_limit);
        self
    }

    /// Sets the option for file data chunk deduplication.
    ///
    /// This option indicates whether data chunk should be deduped when
    /// writing data to a file. It will fall back to repository's
    /// [`dedup_chunk`] if it is not set.
    ///
    /// [`dedup_chunk`]: struct.RepoOpener.html#method.dedup_chunk
    pub fn dedup_chunk(&mut self, dedup_chunk: bool) -> &mut OpenOptions {
        self.dedup_chunk = Some(dedup_chunk);
        self
    }

    /// Opens a file at path with the options specified by `self`.
    pub fn open<P: AsRef<Path>>(&self, repo: &mut Repo, path: P) -> Result<File> {
        // version limit must be greater than 0
        if let Some(version_limit) = self.version_limit {
            if version_limit == 0 {
                return Err(Error::InvalidArgument);
            }
        }
        match repo.fs {
            Some(ref mut fs) => open_file_with_options(fs, path, self),
            None => Err(Error::Closed),
        }
    }
}

/// Information about a repository.
#[derive(Debug)]
pub struct RepoInfo {
    volume_id: Eid,
    ver: base::Version,
    uri: String,
    cost: Cost,
    cipher: Cipher,
    compress: bool,
    version_limit: u8,
    dedup_chunk: bool,
    read_only: bool,
    ctime: Time,
}

impl RepoInfo {
    /// Returns the unique volume id in this repository.
    #[inline]
    pub fn volume_id(&self) -> &Eid {
        &self.volume_id
    }

    /// Returns repository version string.
    ///
    /// This is the string representation of this repository, for example,
    /// `1.0.2`.
    #[inline]
    pub fn version(&self) -> String {
        self.ver.to_string()
    }

    /// Returns the location URI string of this repository.
    #[inline]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns the operation limit for repository password hash.
    #[inline]
    pub fn ops_limit(&self) -> OpsLimit {
        self.cost.ops_limit
    }

    /// Returns the memory limit for repository password hash
    #[inline]
    pub fn mem_limit(&self) -> MemLimit {
        self.cost.mem_limit
    }

    /// Returns repository password encryption cipher.
    #[inline]
    pub fn cipher(&self) -> Cipher {
        self.cipher
    }

    /// Returns whether compression is enabled.
    #[inline]
    pub fn compress(&self) -> bool {
        self.compress
    }

    /// Returns the default maximum number of file versions.
    #[inline]
    pub fn version_limit(&self) -> u8 {
        self.version_limit
    }

    /// Returns whether the file data chunk deduplication is enabled.
    #[inline]
    pub fn dedup_chunk(&self) -> bool {
        self.dedup_chunk
    }

    /// Returns whether this repository is read-only.
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Returns the creation time of this repository.
    #[inline]
    pub fn created_at(&self) -> SystemTime {
        self.ctime.to_system_time()
    }
}

// open a regular file with options
fn open_file_with_options<P: AsRef<Path>>(
    fs: &mut Fs,
    path: P,
    open_opts: &OpenOptions,
) -> Result<File> {
    if fs.is_read_only()
        && (open_opts.write
            || open_opts.append
            || open_opts.truncate
            || open_opts.create
            || open_opts.create_new)
    {
        return Err(Error::ReadOnly);
    }

    let path = path.as_ref();

    match fs.resolve(path) {
        Ok(_) => {
            if open_opts.create_new {
                return Err(Error::AlreadyExists);
            }
        }
        Err(ref err) if *err == Error::NotFound && open_opts.create => {
            let mut opts = fs.get_opts();
            if let Some(version_limit) = open_opts.version_limit {
                opts.version_limit = version_limit;
            }
            if let Some(dedup_chunk) = open_opts.dedup_chunk {
                opts.dedup_chunk = dedup_chunk;
            }
            fs.create_fnode(path, FileType::File, opts)?;
        }
        Err(err) => return Err(err),
    }

    let curr_len;
    let handle = fs.open_fnode(path)?;
    {
        let fnode = handle.fnode.read().unwrap();
        if fnode.is_dir() {
            return Err(Error::IsDir);
        }
        curr_len = fnode.curr_len();
    }

    let pos = if open_opts.append {
        SeekFrom::Start(curr_len as u64)
    } else {
        SeekFrom::Start(0)
    };
    let mut file = File::new(handle, pos, open_opts.read, open_opts.write);

    if open_opts.truncate && curr_len > 0 {
        file.set_len(0)?;
    }

    Ok(file)
}

/// An encrypted repository contains the whole file system.
///
/// A `Repo` represents a secure collection which consists of files,
/// directories and their associated data. Similar to [`std::fs`], `Repo`
/// provides methods to manipulate the enclosed file system.
///
/// # Create and open `Repo`
///
/// `Repo` can be created on different storages using [`RepoOpener`]. It uses
/// an URI-like string to specify its location. Supported storages are listed
/// below:
///
/// * OS file system based storage, location prefix: `file://`
/// * Memory based storage, location prefix: `mem://`
/// * SQLite based storage, location prefix: `sqlite://`
/// * Redis based storage, location prefix: `redis://`
///
/// Check details at: [RepoOpener](struct.RepoOpener.html#method.open)
///
/// `Repo` can only be opened once at a time. After opened, it keeps locked
/// from other open attempts until it goes out scope.
///
/// Optionally, `Repo` can also be opened in [`read-only`] mode.
///
/// # Examples
///
/// Create an OS file system based repository.
///
/// ```no_run
/// # #![allow(unused_mut, unused_variables, dead_code)]
/// # use zbox::Result;
/// use zbox::{init_env, RepoOpener};
///
/// # fn foo() -> Result<()> {
/// init_env();
/// let mut repo = RepoOpener::new()
///     .create(true)
///     .open("file:///path/to/repo", "pwd")?;
/// # Ok(())
/// # }
/// ```
///
/// Create a memory based repository.
///
/// ```
/// # #![allow(unused_mut, unused_variables, dead_code)]
/// # use zbox::{init_env, Result, RepoOpener};
/// # fn foo() -> Result<()> {
/// # init_env();
/// let mut repo = RepoOpener::new().create(true).open("mem://foo", "pwd")?;
/// # Ok(())
/// # }
/// ```
///
/// Open a repository in read-only mode.
///
/// ```no_run
/// # #![allow(unused_mut, unused_variables, dead_code)]
/// # use zbox::{Result, RepoOpener};
/// # fn foo() -> Result<()> {
/// let mut repo = RepoOpener::new()
///     .read_only(true)
///     .open("file:///path/to/repo", "pwd")?;
/// # Ok(())
/// # }
/// ```
///
/// [`std::fs`]: https://doc.rust-lang.org/std/fs/index.html
/// [`init_env`]: fn.init_env.html
/// [`RepoOpener`]: struct.RepoOpener.html
/// [`read-only`]: struct.RepoOpener.html#method.read_only
pub struct Repo {
    fs: Option<Fs>,
}

impl Repo {
    /// Returns whether the URI points at an existing repository.
    ///
    /// Existence check depends on the underlying storage implementation, for
    /// memory storage, it always returns false. For file storage, it will
    /// return if the specified path exists on the OS file system.
    #[inline]
    pub fn exists(uri: &str) -> Result<bool> {
        Fs::exists(uri)
    }

    // create repo
    #[inline]
    fn create(uri: &str, pwd: &str, cfg: &Config) -> Result<Repo> {
        let fs = Fs::create(uri, pwd, cfg)?;
        Ok(Repo { fs: Some(fs) })
    }

    // open repo
    #[inline]
    fn open(uri: &str, pwd: &str, read_only: bool) -> Result<Repo> {
        let fs = Fs::open(uri, pwd, read_only)?;
        Ok(Repo { fs: Some(fs) })
    }

    // close repo
    #[inline]
    pub fn close(&mut self) -> Result<()> {
        match self.fs.take() {
            Some(mut fs) => fs.close(),
            None => Ok(()),
        }
    }

    /// Get repository metadata infomation.
    pub fn info(&self) -> Result<RepoInfo> {
        match self.fs {
            Some(ref fs) => {
                let meta = fs.info();
                Ok(RepoInfo {
                    volume_id: meta.vol_info.id.clone(),
                    ver: meta.vol_info.ver.clone(),
                    uri: meta.vol_info.uri.clone(),
                    cost: meta.vol_info.cost.clone(),
                    cipher: meta.vol_info.cipher.clone(),
                    compress: meta.vol_info.compress,
                    version_limit: meta.opts.version_limit,
                    dedup_chunk: meta.opts.dedup_chunk,
                    read_only: meta.read_only,
                    ctime: meta.vol_info.ctime.clone(),
                })
            }
            None => Err(Error::Closed),
        }
    }

    /// Reset password for the respository.
    pub fn reset_password(
        &mut self,
        old_pwd: &str,
        new_pwd: &str,
        ops_limit: OpsLimit,
        mem_limit: MemLimit,
    ) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => {
                let cost = Cost::new(ops_limit, mem_limit);
                fs.reset_password(old_pwd, new_pwd, cost)
            }
            None => Err(Error::Closed),
        }
    }

    /// Returns whether the path points at an existing entity in repository.
    ///
    /// `path` must be an absolute path.
    pub fn path_exists<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        match self.fs {
            Some(ref fs) => Ok(fs.resolve(path.as_ref()).map(|_| true).unwrap_or(false)),
            None => Err(Error::Closed),
        }
    }

    /// Returns whether the path exists in repository and is pointing at
    /// a regular file.
    ///
    /// `path` must be an absolute path.
    pub fn is_file<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        match self.fs {
            Some(ref fs) => match fs.resolve(path.as_ref()) {
                Ok(fnode_ref) => {
                    let fnode = fnode_ref.read().unwrap();
                    Ok(fnode.is_file())
                }
                Err(_) => Ok(false),
            },
            None => Err(Error::Closed),
        }
    }

    /// Returns whether the path exists in repository and is pointing at
    /// a directory.
    ///
    /// `path` must be an absolute path.
    pub fn is_dir<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        match self.fs {
            Some(ref fs) => match fs.resolve(path.as_ref()) {
                Ok(fnode_ref) => {
                    let fnode = fnode_ref.read().unwrap();
                    Ok(fnode.is_dir())
                }
                Err(_) => Ok(false),
            },
            None => Err(Error::Closed),
        }
    }

    /// Create a file in read-write mode.
    ///
    /// This function will create a file if it does not exist, and will
    /// truncate it if it does.
    ///
    /// See the [`OpenOptions::open`](struct.OpenOptions.html#method.open)
    /// function for more details.
    #[inline]
    pub fn create_file<P: AsRef<Path>>(&mut self, path: P) -> Result<File> {
        if self.fs.is_none() {
            return Err(Error::Closed);
        }
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .open(self, path)
    }

    /// Attempts to open a file in read-only mode.
    ///
    /// `path` must be an absolute path.
    ///
    /// See the [`OpenOptions::open`] function for more details.
    ///
    /// # Errors
    /// This function will return an error if path does not already exist.
    /// Other errors may also be returned according to [`OpenOptions::open`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut, unused_variables, dead_code)]
    /// # use zbox::{init_env, Result, RepoOpener};
    /// # fn foo() -> Result<()> {
    /// # init_env();
    /// # let mut repo = RepoOpener::new()
    /// #     .create(true)
    /// #     .open("mem://foo", "pwd")?;
    /// let mut f = repo.open_file("foo.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`OpenOptions::open`]: struct.OpenOptions.html#method.open
    #[inline]
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<File> {
        if self.fs.is_none() {
            return Err(Error::Closed);
        }
        OpenOptions::new().open(self, path)
    }

    /// Creates a new, empty directory at the specified path.
    ///
    /// `path` must be an absolute path.
    #[inline]
    pub fn create_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => fs
                .create_fnode(path.as_ref(), FileType::Dir, Options::default())
                .map(|_| ()),
            None => Err(Error::Closed),
        }
    }

    /// Recursively create a directory and all of its parent components if they
    /// are missing.
    ///
    /// `path` must be an absolute path.
    #[inline]
    pub fn create_dir_all<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => fs.create_dir_all(path.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Returns a vector of all the entries within a directory.
    ///
    /// `path` must be an absolute path.
    #[inline]
    pub fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DirEntry>> {
        match self.fs {
            Some(ref fs) => fs.read_dir(path.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Given a path, query the repository to get information about a file,
    /// directory, etc.
    ///
    /// `path` must be an absolute path.
    #[inline]
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Metadata> {
        match self.fs {
            Some(ref fs) => fs.metadata(path.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Return a vector of history versions of a regular file.
    ///
    /// `path` must be an absolute path to a regular file.
    #[inline]
    pub fn history<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Version>> {
        match self.fs {
            Some(ref fs) => fs.history(path.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Copies the content of one file to another.
    ///
    /// This function will overwrite the content of `to`.
    ///
    /// If `from` and `to` both point to the same file, then this function will
    /// do nothing.
    ///
    /// `from` and `to` must be absolute paths to regular files.
    #[inline]
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&mut self, from: P, to: Q) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => fs.copy(from.as_ref(), to.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Removes a regular file from the repository.
    ///
    /// `path` must be an absolute path.
    #[inline]
    pub fn remove_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => fs.remove_file(path.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Remove an existing empty directory.
    ///
    /// `path` must be an absolute path.
    #[inline]
    pub fn remove_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => fs.remove_dir(path.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Removes a directory at this path, after removing all its children.
    /// Use carefully!
    ///
    /// `path` must be an absolute path.
    #[inline]
    pub fn remove_dir_all<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => fs.remove_dir_all(path.as_ref()),
            None => Err(Error::Closed),
        }
    }

    /// Rename a file or directory to a new name, replacing the original file
    /// if to already exists.
    ///
    /// `from` and `to` must be absolute paths.
    #[inline]
    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&mut self, from: P, to: Q) -> Result<()> {
        match self.fs {
            Some(ref mut fs) => fs.rename(from.as_ref(), to.as_ref()),
            None => Err(Error::Closed),
        }
    }
}

impl Debug for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Repo").finish()
    }
}