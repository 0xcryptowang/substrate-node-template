# Substrate Node Template 模版项目，添加poe存证模块 

**与3.0相比，常见的修改为：**
1. Substrate 4.0的发布时间没有确定，我们开发时使用git的分支版本，比如

```toml
frame-support = { default-features = false, git = 'https://github.com/paritytech/substrate.git', tag = 'devhub/latest', version = '4.0.0-dev' }
```

这里的 devhub/latest 替换为node-templat中所对应的版本。

2. 新增了`scale-info`包，在模块的Cargo.toml里引入，它会自动注册 runtime 模块中使用的函数参数和存储项的类型信息，从而前端在调用模块时无需再注册类型信息。

```toml
scale-info = { default-features = false, features = ['derive'], version = '1.0' }
```

在模块代码lib.rs中，删除此类定义：
```rust
#[pallet::metadata(T::AccountId = "AccountId")]
```

3. 在mock.rs，中宏所生成的模块结构体名由Module变成Pallet，即

```rust=
frame_support::construct_runtime!(
	// -- snip --
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		// -- snip --
	}
);
```


4. 在mock.rs中，system 模块BaseCallFilter不再使用`()`来过滤可调用函数，而是使用`frame_support::traits::Everything`。

```rust=
impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	// -- snip --
}
```

