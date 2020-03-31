const addon = require('../native');

module.exports = {
    decode: function(str) {
        return new Promise(function(resolve, reject) {
            addon.decode(str, function(error, result) {
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
