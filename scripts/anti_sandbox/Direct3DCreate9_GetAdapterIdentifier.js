(() => {
  const TAG = "anti_sandbox";
  const API_NAME = "Direct3DCreate9";
  const MODULE_NAME = "d3d9.dll";

  const D3D_SDK_VERSION = 32;
  const D3D_VENDOR_NVIDIA = 0x10de;

  const VTABLE_GET_ADAPTER_COUNT = 4;
  const VTABLE_GET_ADAPTER_IDENTIFIER = 5;

  // D3DADAPTER_IDENTIFIER9:
  // Driver[512], Description[512], DeviceName[32], DriverVersion[8], VendorId[4]
  const D3DADAPTER_IDENTIFIER9_VENDOR_ID_OFFSET = 512 + 512 + 32 + 8;

  const ARG_SPEC = [
    { index: 0, name: "SDKVersion" },
  ];
  const GET_ADAPTER_COUNT_ARG_SPEC = [
    { index: 0, name: "this" },
  ];
  const GET_ADAPTER_IDENTIFIER_ARG_SPEC = [
    { index: 0, name: "this" },
    { index: 1, name: "Adapter" },
    { index: 2, name: "Flags" },
    { index: 3, name: "pIdentifier" },
  ];

  const hookedMethods = {};

  function hex(value) {
    return "0x" + value.toString(16);
  }

  function readCStringField(base, offset, length) {
    if (!base || base.isNull()) return "";

    try {
      const field = base.add(offset);
      const text = field.readCString(length);
      return text || "";
    } catch (e) {
      return "";
    }
  }

  function hookVtableMethod(tagName, d3d, index, handlerFactory) {
    const vtable = d3d.readPointer();
    const method = vtable.add(Process.pointerSize * index).readPointer();
    const key = method.toString();

    if (hookedMethods[key]) {
      return method;
    }

    hookedMethods[key] = true;
    Interceptor.attach(method, handlerFactory(method));

    Agent.register(TAG, {
      apiName: tagName,
      moduleName: MODULE_NAME,
      address: method.toString(),
      vtableIndex: index,
    });

    return method;
  }

  function hookD3D9Object(d3d) {
    hookVtableMethod("IDirect3D9::GetAdapterCount", d3d, VTABLE_GET_ADAPTER_COUNT, () => ({
      onEnter(args) {
        Agent.collect(TAG, args, GET_ADAPTER_COUNT_ARG_SPEC, {
          apiName: "IDirect3D9::GetAdapterCount",
          moduleName: MODULE_NAME,
        });
      },

      onLeave(retval) {
        Agent.triggered(TAG, {
          apiName: "IDirect3D9::GetAdapterCount",
          moduleName: MODULE_NAME,
          adapterCount: retval.toUInt32(),
        });
      },
    }));

    hookVtableMethod("IDirect3D9::GetAdapterIdentifier", d3d, VTABLE_GET_ADAPTER_IDENTIFIER, () => ({
      onEnter(args) {
        Agent.collect(TAG, args, GET_ADAPTER_IDENTIFIER_ARG_SPEC, {
          apiName: "IDirect3D9::GetAdapterIdentifier",
          moduleName: MODULE_NAME,
        });

        this.adapter = args[1].toUInt32();
        this.flags = args[2].toUInt32();
        this.identifier = args[3];
      },

      onLeave(retval) {
        const hr = retval.toInt32();
        const success = hr >= 0;

        if (!success || !this.identifier || this.identifier.isNull()) {
          Agent.triggered(TAG, {
            apiName: "IDirect3D9::GetAdapterIdentifier",
            moduleName: MODULE_NAME,
            adapter: this.adapter,
            flags: this.flags,
            hresult: hex(hr >>> 0),
            patched: false,
          });

          return;
        }

        const vendorPtr = this.identifier.add(D3DADAPTER_IDENTIFIER9_VENDOR_ID_OFFSET);
        const originalVendorId = vendorPtr.readU32();

        vendorPtr.writeU32(D3D_VENDOR_NVIDIA);

        Agent.patched(TAG, {
          apiName: "IDirect3D9::GetAdapterIdentifier",
          moduleName: MODULE_NAME,
          adapter: this.adapter,
          flags: this.flags,
          hresult: hex(hr >>> 0),
          driver: readCStringField(this.identifier, 0, 512),
          description: readCStringField(this.identifier, 512, 512),
          deviceName: readCStringField(this.identifier, 1024, 32),
          originalVendorId: hex(originalVendorId),
          patchedVendorId: hex(D3D_VENDOR_NVIDIA),
          patched: true,
        });
      },
    }));
  }

  Agent.safeCall(TAG, () => {
    if (!Process.findModuleByName(MODULE_NAME)) {
      Module.load(MODULE_NAME);
    }

    const addr = Agent.getExport(MODULE_NAME, API_NAME);

    if (!addr) {
      Agent.skip(TAG, {
        apiName: API_NAME,
        moduleName: MODULE_NAME,
        reason: "export_not_found",
      });

      return;
    }

    Interceptor.attach(addr, {
      onEnter(args) {
        Agent.collect(TAG, args, ARG_SPEC, {
          apiName: API_NAME,
          moduleName: MODULE_NAME,
          expectedSdkVersion: D3D_SDK_VERSION,
        });
      },

      onLeave(retval) {
        Agent.triggered(TAG, {
          apiName: API_NAME,
          moduleName: MODULE_NAME,
          returnedObject: retval.toString(),
          returnedNull: retval.isNull(),
        });

        if (!retval.isNull()) {
          hookD3D9Object(retval);
        }
      },
    });

    Agent.register(TAG, {
      apiName: API_NAME,
      moduleName: MODULE_NAME,
      address: addr.toString(),
    });
  });
})();
