const addon = require('../native');

module.exports = {
    decode: function(str, max_decompressed_size) {
        return new Promise(function(resolve, reject) {
            addon.decode(str, max_decompressed_size, function(error, result) {
                if (error) {
                    return reject(error);
                }

                try {
                    const obj = JSON.parse(result);
                    resolve(obj);
                } catch (error) {
                    reject(error);
                }
            });
        });
    },
    encode: function(value) {
        try {
            const stringified = JSON.stringify(value);
            return new Promise(function(resolve, reject) {
                addon.encode(stringified, function(error, result) {
                    if (error) {
                        return reject(error);
                    }
                    resolve(result);
                });
            });
        } catch (error) {
            return Promise.reject(error);
        }
    },
    decodeSync: function(str, max_decompressed_size) {
        return JSON.parse(addon.decodeSync(str, max_decompressed_size));
    },
    encodeSync: function(obj) {
        return addon.encodeSync(JSON.stringify(obj));
    },
};
