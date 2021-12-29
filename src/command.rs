use async_process::unix::CommandExt as _;

pub struct Command {
    inner: async_process::Command,
    stdin: Option<std::process::Stdio>,
    stdout: Option<std::process::Stdio>,
    stderr: Option<std::process::Stdio>,
}

impl Command {
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: async_process::Command::new(program),
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    pub fn env_remove<K: AsRef<std::ffi::OsStr>>(
        &mut self,
        key: K,
    ) -> &mut Self {
        self.inner.env_remove(key);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.inner.env_clear();
        self
    }

    pub fn current_dir<P: AsRef<std::path::Path>>(
        &mut self,
        dir: P,
    ) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn stdin<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: Option<T>,
    ) -> &mut Self {
        self.stdin = cfg.map(Into::into);
        self
    }

    pub fn stdout<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: Option<T>,
    ) -> &mut Self {
        self.stdout = cfg.map(Into::into);
        self
    }

    pub fn stderr<T: Into<std::process::Stdio>>(
        &mut self,
        cfg: Option<T>,
    ) -> &mut Self {
        self.stderr = cfg.map(Into::into);
        self
    }

    pub fn spawn(&mut self, pty: crate::Pty) -> crate::Result<Child> {
        let (stdin, stdout, stderr, pre_exec) =
            crate::sys::setup_subprocess(&pty, pty.pts()?)?;

        self.inner.stdin(self.stdin.take().unwrap_or(stdin));
        self.inner.stdout(self.stdout.take().unwrap_or(stdout));
        self.inner.stderr(self.stderr.take().unwrap_or(stderr));

        // safe because setsid() and close() are async-signal-safe functions
        // and ioctl() is a raw syscall (which is inherently
        // async-signal-safe).
        unsafe { self.inner.pre_exec(pre_exec) };

        let child = self.inner.spawn()?;

        Ok(Child::new(child, pty))
    }
}

pub struct Child {
    inner: async_process::Child,
    pty: crate::Pty,
}

impl Child {
    fn new(inner: async_process::Child, pty: crate::Pty) -> Self {
        Self { inner, pty }
    }

    #[must_use]
    pub fn pty(&self) -> &crate::Pty {
        &self.pty
    }

    #[must_use]
    pub fn pty_mut(&mut self) -> &mut crate::Pty {
        &mut self.pty
    }
}

impl std::ops::Deref for Child {
    type Target = async_process::Child;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Child {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
