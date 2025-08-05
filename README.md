### Rust Single

用来确保应用程序单进程运行.

### 使用

```rust

fn create_single(path: PathBuf, info: &str) -> AnyResult<Option<Single>> {
    let single = Single::create(path, info)?;
    if !single.is_single {
        log::error!("存在已启动进程: {}", single.pid.unwrap_or(0));
        log::error!("已启动进程info: {}", single.info);
        Err(Box::new(BizError::SingleRunning))
    } else {
        Ok(Some(single))
    }
}

fn main() ->AnyResult<()> {
    let lock_path = "/tmp/single.lock";
    let mut o_single = create_single(lock_path, "single info")?;
    
    // on close
    let o = o_single.take();
    if let Some(single) = o {
        // 如果要同时兼容 root和非root权限启动, 需要在释放后删除文件. 避免由于权限导致异常
        let path = single.path.clone();
        let path_info = single.path_info.clone();
        drop(single);
        let _ = file::delete(&path);
        let _ = file::delete(&path_info);
    }
}

```
