globalThis.Agent = {
  modules: {},
  moduleCallbacks: {},
  attachedApis: {},

  // Event output API.
  log(event, tag, subject = null, data = {}) {
    send({
      schema: "argus.frida.v1",
      time: new Date().toISOString(),
      event,
      tag,
      subject: subject || {
        name: tag,
        address: null,
      },
      data: data || {},
    });
  },
  apiSubject(moduleName, apiName, address = null) {
    return {
      name: `${moduleName}!${apiName}`,
      address: address ? address.toString() : null,
    };
  },

  moduleSubject(moduleName, address = null) {
    return {
      name: moduleName,
      address: address ? address.toString() : null,
    };
  },

  collectArgs(args, spec) {
    const items = {};
    for (const item of spec) {
      items[item.name] = args[item.index].toString();
    }
    return items;
  },

  init(tag, subject = null, data = {}) {
    this.log("init", tag, subject, data);
  },

  skip(tag, subject = null, data = {}) {
    this.log("skip", tag, subject, data);
  },

  register(tag, moduleName, apiName) {
    this.log("register", tag, this.apiSubject(moduleName, apiName));
  },
  collect(tag, moduleName, apiName, caller, args, spec) {
    this.log("collect", tag, this.apiSubject(moduleName, apiName, caller), {
      args: this.collectArgs(args, spec),
    });
  },
  triggered(tag, moduleName, apiName, caller, data = {}) {
    this.log(
      "triggered",
      tag,
      this.apiSubject(moduleName, apiName, caller),
      data,
    );
  },
  error(tag, subject, message, data = {}) {
    this.log("error", tag, subject, {
      message: String(message),
      ...data,
    });
  },

  // Rule SDK.
  whenModuleLoaded(moduleName, callback) {
    const key = this.normalizeModuleName(moduleName);
    const existing = Process.findModuleByName(key);

    if (existing) {
      this.modules[key] = existing;
      callback(existing);
      return;
    }

    if (!this.moduleCallbacks[key]) {
      this.moduleCallbacks[key] = [];
    }

    this.moduleCallbacks[key].push(callback);

    this.skip("module", this.moduleSubject(key), {
      moduleName: key,
      reason: "pending_module_load",
    });
  },

  attachApi(tag, moduleName, apiName, handlerFactory) {
    const normalizedModuleName = this.normalizeModuleName(moduleName);
    const key = `${normalizedModuleName}!${apiName}`;

    if (this.attachedApis[key]) {
      return false;
    }

    const addr = this.getExport(normalizedModuleName, apiName);

    if (!addr) {
      return false;
    }

    this.attachedApis[key] = true;
    Interceptor.attach(addr, handlerFactory(addr));
    this.register(tag, normalizedModuleName, apiName);

    return true;
  },

  attachApis(tag, hooks, handlerFactory) {
    for (const hook of hooks) {
      this.whenModuleLoaded(hook.moduleName, () => {
        this.attachApi(tag, hook.moduleName, hook.apiName, (addr) =>
          handlerFactory(hook, addr),
        );
      });
    }
  },

  // Bootstrap lifecycle.
  initMainModule() {
    const m = Process.enumerateModules()[0];

    if (!m) {
      this.skip("bootstrap", this.moduleSubject("main"), {
        reason: "main_module_not_found",
      });
      return;
    }

    this.init("bootstrap", this.moduleSubject(m.name, m.base), {
      moduleName: m.name,
      base: m.base.toString(),
      size: String(m.size),
      path: m.path,
    });
  },
  initModules() {
    const names = ["ntdll.dll", "kernel32.dll", "kernelbase.dll", "d3d9.dll"];

    for (const name of names) {
      const m = Process.findModuleByName(name);

      if (m) {
        this.modules[name.toLowerCase()] = m;

        this.init("bootstrap", this.moduleSubject(name, m.base), {
          moduleName: name,
          base: m.base.toString(),
        });
      } else {
        this.skip("bootstrap", this.moduleSubject(name), {
          moduleName: name,
        });
      }
    }
  },

  notifyModuleLoaded(moduleName) {
    const key = this.normalizeModuleName(moduleName);

    if (!key) {
      return;
    }

    const m = Process.findModuleByName(key);

    if (!m) {
      return;
    }

    this.modules[key] = m;

    const callbacks = this.moduleCallbacks[key] || [];
    delete this.moduleCallbacks[key];

    if (callbacks.length === 0) {
      return;
    }

    this.init("module", this.moduleSubject(m.name, m.base), {
      moduleName: m.name,
      base: m.base.toString(),
      lateLoaded: true,
    });

    for (const callback of callbacks) {
      this.safeCall(`module:${key}`, () => callback(m));
    }
  },

  getModule(name) {
    const key = this.normalizeModuleName(name);

    if (this.modules[key]) {
      return this.modules[key];
    }

    const m = Process.findModuleByName(key);

    if (m) {
      this.modules[key] = m;

      this.init("bootstrap", this.moduleSubject(m.name, m.base), {
        moduleName: m.name,
        base: m.base.toString(),
        lateLoaded: true,
      });

      return m;
    }

    this.skip("module", this.moduleSubject(key), {
      moduleName: key,
    });

    return null;
  },

  getExport(moduleName, exportName) {
    let addr = null;
    const normalizedModuleName = this.normalizeModuleName(moduleName);

    try {
      const m = Process.findModuleByName(normalizedModuleName);

      if (m && typeof m.findExportByName === "function") {
        addr = m.findExportByName(exportName);
      } else if (m && typeof m.getExportByName === "function") {
        addr = m.getExportByName(exportName);
      }
    } catch (e) {
      addr = null;
    }

    if (!addr) {
      this.error(
        "export",
        this.apiSubject(normalizedModuleName, exportName),
        "export not found",
        {
          moduleName: normalizedModuleName,
          apiName: exportName,
          api: `${normalizedModuleName}!${exportName}`,
        },
      );

      return null;
    }

    return addr;
  },

  mustGetExport(moduleName, exportName) {
    return Module.getExportByName(moduleName, exportName);
  },

  hookModuleLoader() {
    const moduleName = "ntdll.dll";
    const apiName = "LdrLoadDll";
    const addr = this.getExport(moduleName, apiName);

    if (!addr) {
      return;
    }

    Interceptor.attach(addr, {
      onEnter(args) {
        this.caller = this.returnAddress;
        this.dllName = Agent.readUnicodeString(args[2]);
      },

      onLeave(retval) {
        const status = retval.toInt32();

        if (status !== 0 || !this.dllName) {
          return;
        }

        if (!Agent.hasModuleCallbacks(this.dllName)) {
          return;
        }

        Agent.triggered(
          "module_loader",
          moduleName,
          apiName,
          this.caller.toString(),
          {
            original: { moduleName: Agent.normalizeModuleName(this.dllName) },
            current: { moduleName: Agent.normalizeModuleName(this.dllName) },
          },
        );

        Agent.notifyModuleLoaded(this.dllName);
      },
    });

    this.register("module_loader", moduleName, apiName);
  },

  safeCall(tag, fn) {
    try {
      return fn();
    } catch (e) {
      this.error(tag, { name: tag, address: null }, e, {
        stack: e.stack || null,
      });

      return null;
    }
  },

  // General helpers.
  normalizeModuleName(name) {
    return String(name || "")
      .split("\\")
      .pop()
      .split("/")
      .pop()
      .toLowerCase();
  },

  hasModuleCallbacks(moduleName) {
    const key = this.normalizeModuleName(moduleName);
    const callbacks = this.moduleCallbacks[key];
    return !!callbacks && callbacks.length > 0;
  },

  readUtf16(ptr) {
    if (!ptr || ptr.isNull()) return "";
    return ptr.readUtf16String();
  },

  readAnsi(ptr) {
    if (!ptr || ptr.isNull()) return "";
    return ptr.readCString();
  },

  ptrKey(ptrValue) {
    return ptrValue ? ptrValue.toString() : "";
  },

  containsAny(value, keywords) {
    const normalized = String(value || "").toLowerCase();

    for (const keyword of keywords) {
      if (normalized.includes(String(keyword).toLowerCase())) {
        return true;
      }
    }

    return false;
  },

  readBstr(ptrValue) {
    if (!ptrValue || ptrValue.isNull()) return "";

    try {
      return ptrValue.readUtf16String();
    } catch (_) {
      return "";
    }
  },

  readString(ptrValue, wide) {
    if (!ptrValue || ptrValue.isNull()) return "";

    try {
      return wide ? ptrValue.readUtf16String() : ptrValue.readAnsiString();
    } catch (_) {
      return "";
    }
  },

  writeString(ptrValue, maxChars, value, wide) {
    if (!ptrValue || ptrValue.isNull() || maxChars <= 0) {
      return 0;
    }

    const text = String(value || "").slice(0, Math.max(0, maxChars - 1));

    if (wide) {
      ptrValue.writeUtf16String(text);
    } else {
      ptrValue.writeAnsiString(text);
    }

    return text.length;
  },

  readUnicodeString(ptrValue) {
    if (!ptrValue || ptrValue.isNull()) return "";

    try {
      const length = ptrValue.readU16();
      const bufferOffset = Process.pointerSize === 8 ? 8 : 4;
      const buffer = ptrValue.add(bufferOffset).readPointer();

      if (!buffer || buffer.isNull() || length === 0) {
        return "";
      }

      return buffer.readUtf16String(length / 2);
    } catch (_) {
      return "";
    }
  },

  comMethod(comObject, index) {
    return comObject.readPointer().add(Process.pointerSize * index).readPointer();
  },

  isGuid(ptrValue, bytes) {
    if (!ptrValue || ptrValue.isNull() || !bytes || bytes.length !== 16) {
      return false;
    }

    try {
      for (let i = 0; i < 16; i++) {
        if (ptrValue.add(i).readU8() !== bytes[i]) {
          return false;
        }
      }

      return true;
    } catch (_) {
      return false;
    }
  },

  sysAllocString(value) {
    if (!this._sysAllocString) {
      this._sysAllocString = new NativeFunction(
        this.mustGetExport("oleaut32.dll", "SysAllocString"),
        "pointer",
        ["pointer"],
      );
    }

    return this._sysAllocString(Memory.allocUtf16String(String(value || "")));
  },

  writeVariantU32(variant, value) {
    if (!variant || variant.isNull()) return;

    variant.writeU16(19);
    variant.add(8).writeU32(value);
  },

  writeVariantU64(variant, value) {
    if (!variant || variant.isNull()) return;

    variant.writeU16(21);
    variant.add(8).writeU64(new UInt64(String(value)));
  },

  writeVariantBstr(variant, value) {
    if (!variant || variant.isNull()) return;

    variant.writeU16(8);
    variant.add(8).writePointer(this.sysAllocString(value));
  },

  writeVariantAuto(variant, value) {
    if (!variant || variant.isNull()) return;

    const vt = variant.readU16();

    if (vt === 3 || vt === 19) {
      this.writeVariantU32(variant, Number(value));
      return;
    }

    if (vt === 20 || vt === 21) {
      this.writeVariantU64(variant, value);
      return;
    }

    this.writeVariantBstr(variant, value);
  },
};

Agent.initMainModule();
Agent.initModules();
Agent.hookModuleLoader();

Agent.init("bootstrap", null, {
  status: "initialized",
});
