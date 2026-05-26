globalThis.Agent = {
  modules: {},

  log(event, tag, extra = null) {
    const record = {
      time: new Date().toISOString(),
      event: event,
      tag: tag,
    };

    if (extra !== null && extra !== undefined) {
      record.extra = extra;
    }

    send(record);
  },
  collectArgs(args, spec) {
    const items = [];

    for (const item of spec) {
      const index = item.index;

      items.push({
        index: index,
        name: item.name,
        value: args[index].toString(),
      });
    }

    return items;
  },

  init(tag, extra = null) {
    this.log("init", tag, extra);
  },

  register(tag, extra = null) {
    this.log("register", tag, extra);
  },

  collect(tag, args, spec, extra = null) {
    this.log("collect", tag, {
      ...(extra || {}),
      args: this.collectArgs(args, spec),
    });
  },

  triggered(tag, extra = null) {
    this.log("triggered", tag, extra);
  },

  patched(tag, extra = null) {
    this.log("patched", tag, extra);
  },

  skip(tag, extra = null) {
    this.log("skip", tag, extra);
  },

  error(tag, extra = null) {
    this.log("error", tag, extra);
  },

  initModules() {
    const names = ["ntdll.dll", "kernel32.dll", "kernelbase.dll", "d3d9.dll"];

    for (const name of names) {
      const m = Process.findModuleByName(name);

      if (m) {
        this.modules[name.toLowerCase()] = m;

        this.init("bootstrap", {
          moduleName: name,
          base: m.base.toString(),
        });
      } else {
        this.skip("bootstrap", {
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

      this.init("bootstrap", {
        moduleName: name,
        base: m.base.toString(),
        lateLoaded: true,
      });

      return m;
    }

    this.skip("module", {
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
      this.error("export", {
        moduleName: moduleName,
        apiName: exportName,
        api: `${moduleName}!${exportName}`,
      });

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
      this.error(tag, {
        error: String(e),
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

Agent.init("bootstrap", {
  status: "initialized",
});
