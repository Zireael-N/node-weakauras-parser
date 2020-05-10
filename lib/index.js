const addon = require('../native');

module.exports = {
    decode: function(str, max_decompressed_size) {
        return new Promise(function(resolve, reject) {
            addon.decode(str, max_decompressed_size, function(error, result) {
                if (error) {
                    return reject(error);
                }
                resolve(result);
            });
        });
    },
    encode: function(value) {
        return new Promise(function(resolve, reject) {
            addon.encode(value, function(error, result) {
                if (error) {
                    return reject(error);
                }
                resolve(result);
            });
        });
    },
    decodeSync: addon.decodeSync,
    encodeSync: addon.encodeSync,
};

//module.exports = addon;
