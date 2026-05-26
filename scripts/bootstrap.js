globalThis.Agent = {
  modules: {},

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

  getModule(name) {
    const key = name.toLowerCase();

    if (this.modules[key]) {
      return this.modules[key];
    }

    const m = Process.findModuleByName(name);

    if (m) {
      this.modules[key] = m;

      this.init("bootstrap", this.moduleSubject(name, m.base), {
        moduleName: name,
        base: m.base.toString(),
        lateLoaded: true,
      });

      return m;
    }

    this.skip("module", this.moduleSubject(name), {
      moduleName: name,
    });

    return null;
  },

  getExport(moduleName, exportName) {
    let addr = null;

    try {
      const m = Process.findModuleByName(moduleName);

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
        this.apiSubject(moduleName, exportName),
        "export not found",
        {
          moduleName,
          apiName: exportName,
          api: `${moduleName}!${exportName}`,
        },
      );

      return null;
    }

    return addr;
  },

  mustGetExport(moduleName, exportName) {
    return Module.getExportByName(moduleName, exportName);
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

  readUtf16(ptr) {
    if (!ptr || ptr.isNull()) return "";
    return ptr.readUtf16String();
  },

  readAnsi(ptr) {
    if (!ptr || ptr.isNull()) return "";
    return ptr.readCString();
  },
};

Agent.initModules();

Agent.init("bootstrap", null, {
  status: "initialized",
});
