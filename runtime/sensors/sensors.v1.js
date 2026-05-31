globalThis.ArgusSensorsV1 = globalThis.ArgusSensorsV1 || {
  version: "argus.sensors.v1",
  sensors: {},

  create(name, install = null) {
    const sensor = {
      name,
      handlers: [],
      installed: false,
      installFactory: install,

      use(handler) {
        this.handlers.push(handler);
        return handler;
      },

      emit(ctx) {
        for (const handler of this.handlers) {
          AgentV1.safeCall(`${name}:${handler.name || "handler"}`, () => {
            if (handler.match && !handler.match(ctx)) {
              return;
            }

            if (handler.apply) {
              handler.apply(ctx);
            }
          });
        }
      },

      install() {
        if (this.installed) {
          return;
        }

        if (!this.installFactory) {
          return;
        }

        this.installed = true;
        this.installFactory(this);
      },
    };

    this.sensors[name] = sensor;
    return sensor;
  },

  define(name, install) {
    const sensor = this.sensors[name] || this.create(name);

    sensor.installFactory = install;
    if (sensor.handlers.length > 0) {
      sensor.install();
    }

    return sensor;
  },

  use(name, handler) {
    const sensor = this.sensors[name] || this.create(name);

    const registered = sensor.use(handler);
    sensor.install();
    return registered;
  },

  reportOnce(cache, key) {
    if (cache[key]) {
      return false;
    }

    cache[key] = true;
    return true;
  },
};
