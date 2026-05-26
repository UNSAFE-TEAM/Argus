(() => {
  const TAG = "anti_sandbox";
  const API_NAME = "Direct3DCreate9";
  const MODULE_NAME = "d3d9.dll";

  const D3D_VENDOR_NVIDIA = 0x10de;

  const VTABLE_GET_ADAPTER_IDENTIFIER = 5;

  // D3DADAPTER_IDENTIFIER9:
  // Driver[512], Description[512], DeviceName[32], DriverVersion[8], VendorId[4]
  const D3DADAPTER_IDENTIFIER9_VENDOR_ID_OFFSET = 512 + 512 + 32 + 8;

  const ARG_SPEC = [{ index: 0, name: "SDKVersion" }];
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

  function hookVtableMethod(tagName, d3d, index, handlerFactory) {
    const vtable = d3d.readPointer();
    const method = vtable.add(Process.pointerSize * index).readPointer();
    const key = method.toString();

    if (hookedMethods[key]) {
      return method;
    }

    hookedMethods[key] = true;
    Interceptor.attach(method, handlerFactory(method));

    Agent.register(TAG, MODULE_NAME, tagName);

    return method;
  }

  function hookD3D9Object(d3d) {
    hookVtableMethod(
      "IDirect3D9::GetAdapterIdentifier",
      d3d,
      VTABLE_GET_ADAPTER_IDENTIFIER,
      (method) => ({
        onEnter(args) {
          this.caller = this.returnAddress;

          Agent.collect(
            TAG,
            MODULE_NAME,
            "IDirect3D9::GetAdapterIdentifier",
            this.caller.toString(),
            args,
            GET_ADAPTER_IDENTIFIER_ARG_SPEC,
          );

          this.adapter = args[1].toUInt32();
          this.flags = args[2].toUInt32();
          this.identifier = args[3];
        },

        onLeave(retval) {
          const hr = retval.toInt32();
          const success = hr >= 0;

          if (!success || !this.identifier || this.identifier.isNull()) {
            return;
          }

          const vendorPtr = this.identifier.add(
            D3DADAPTER_IDENTIFIER9_VENDOR_ID_OFFSET,
          );
          const originalVendorId = vendorPtr.readU32();

          vendorPtr.writeU32(D3D_VENDOR_NVIDIA);

          Agent.triggered(
            TAG,
            MODULE_NAME,
            "IDirect3D9::GetAdapterIdentifier",
            this.caller.toString(),
            {
              original: { vendorId: hex(originalVendorId) },
              current: { vendorId: hex(D3D_VENDOR_NVIDIA) },
            },
          );
        },
      }),
    );
  }

  Agent.safeCall(TAG, () => {
    if (!Process.findModuleByName(MODULE_NAME)) {
      Module.load(MODULE_NAME);
    }

    const addr = Agent.getExport(MODULE_NAME, API_NAME);

    if (!addr) {
      Agent.skip(TAG, Agent.apiSubject(MODULE_NAME, API_NAME), {
        reason: "export_not_found",
      });

      return;
    }

    Interceptor.attach(addr, {
      onEnter(args) {
        this.caller = this.returnAddress;

        Agent.collect(
          TAG,
          MODULE_NAME,
          API_NAME,
          this.caller.toString(),
          args,
          ARG_SPEC,
        );
      },

      onLeave(retval) {
        if (!retval.isNull()) {
          hookD3D9Object(retval);
        }
      },
    });

    Agent.register(TAG, MODULE_NAME, API_NAME);
  });
})();
