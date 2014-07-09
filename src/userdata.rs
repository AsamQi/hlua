use ffi;
use Lua;
use CopyReadable;
use ConsumeReadable;
use Pushable;
use LuaTable;
use std::any::Any;

/*fn destructor<T>(_: T) {

}*/

// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
#[experimental]
pub fn push_userdata<T: ::std::any::Any>(data: T, lua: &mut Lua, metatable: |&mut LuaTable|) -> uint {
    let typeid = format!("{}", data.get_type_id());

    let luaDataRaw = unsafe { ffi::lua_newuserdata(lua.lua, ::std::mem::size_of_val(&data) as ::libc::size_t) };
    let luaData: *mut T = unsafe { ::std::mem::transmute(luaDataRaw) };
    unsafe { ::std::ptr::write(luaData, data) };

    // creating a metatable
    unsafe {
        ffi::lua_newtable(lua.lua);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        ffi::lua_settable(lua.lua, -3);

        // index "__gc" call the object's destructor
        // TODO: 
        /*"__gc".push_to_lua(lua);
        destructor::<T>.push_to_lua(lua);
        ffi::lua_settable(lua.lua, -3);*/

        {
            let mut table = ConsumeReadable::read_from_variable(::LoadedVariable { lua: lua, size: 1 }).ok().unwrap();
            metatable(&mut table);
            ::std::mem::forget(table);
        }

        ffi::lua_setmetatable(lua.lua, -2);
    }

    1
}

// TODO: the type must be Send because the Lua context is Send, but this conflicts with &str
#[experimental]
pub fn read_copy_userdata<T: Clone + ::std::any::Any>(lua: &mut Lua, index: ::libc::c_int) -> Option<T> {
    unsafe {
        let dummyMe: &T = ::std::mem::uninitialized();      // TODO: this is very very hacky, I don't even know if it works
        let expectedTypeid = format!("{}", dummyMe.get_type_id());

        let dataPtr = ffi::lua_touserdata(lua.lua, index);
        if dataPtr.is_null() {
            return None;
        }

        if ffi::lua_getmetatable(lua.lua, -1) == 0 {
            return None;
        }

        "__typeid".push_to_lua(lua);
        ffi::lua_gettable(lua.lua, -2);
        if CopyReadable::read_from_lua(lua, -1) != Some(expectedTypeid) {
            return None;
        }
        ffi::lua_pop(lua.lua, -2);

        let data: &T = ::std::mem::transmute(dataPtr);
        Some(data.clone())
    }
}

#[cfg(test)]
mod tests {
    use Lua;

    #[test]
    fn readwrite() {
        #[deriving(Clone)]
        struct Foo;
        impl<'a> ::Pushable<'a> for Foo {
            fn push_to_lua(self, lua: &mut Lua<'a>) -> uint {
                ::userdata::push_userdata(self, lua, |_|{})
            }
        }
        impl ::CopyReadable for Foo {}

        let mut lua = Lua::new();

        lua.set("a", Foo);
       // let x: Foo = lua.get("a").unwrap();
    }

    #[test]
    fn destructor_called() {
        // TODO: 
        /*let called = ::std::sync::Arc::new(::std::sync::Mutex::new(false));

        struct Foo {
            called: ::std::sync::Arc<::std::sync::Mutex<bool>>
        }

        impl Drop for Foo {
            fn drop(&mut self) {
                let mut called = self.called.lock();
                (*called) = true;
            }
        }

        impl<'a> ::Pushable<'a> for Foo {}

        {
            let mut lua = Lua::new();
            lua.set("a", Foo{called: called.clone()});
        }

        let locked = called.lock();
        assert!(*locked);*/
    }

    #[test]
    fn type_check() {
        #[deriving(Clone)]
        struct Foo;
        impl<'a> ::Pushable<'a> for Foo {
            fn push_to_lua(self, lua: &mut Lua<'a>) -> uint {
                ::userdata::push_userdata(self, lua, |_|{})
            }
        }
        impl ::CopyReadable for Foo {}

        #[deriving(Clone)]
        struct Bar;
        impl<'a> ::Pushable<'a> for Bar {
            fn push_to_lua(self, lua: &mut Lua<'a>) -> uint {
                ::userdata::push_userdata(self, lua, |_|{})
            }
        }
        impl ::CopyReadable for Bar {}

        let mut lua = Lua::new();

        lua.set("a", Foo);
        
        /*let x: Option<Bar> = lua.get("a");
        assert!(x.is_none())*/
    }

    #[test]
    fn metatables() {
        #[deriving(Clone)]
        struct Foo;
        impl<'a> ::Pushable<'a> for Foo {
            fn push_to_lua(self, lua: &mut Lua<'a>) -> uint {
                ::userdata::push_userdata(self, lua, |table| {
                    table.set("__index".to_string(), vec!(
                        ("test".to_string(), || 5i)
                    ));
                })
            }
        }

        let mut lua = Lua::new();

        lua.set("a", Foo);

        let x: int = lua.execute("return a.test()").unwrap();
        assert_eq!(x, 5);
    }
}
