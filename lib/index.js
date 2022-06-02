const addon = require('../native');

module.exports = {
    decode: function(str, max_decompressed_size) {
        return addon.decode(str, max_decompressed_size).then(function(result) {
            return JSON.parse(result);
        });
    },
    encode: function(value) {
        return addon.encode(JSON.stringify(value));
    },
    decodeSync: function(str, max_decompressed_size) {
        return JSON.parse(addon.decodeSync(str, max_decompressed_size));
    },
    encodeSync: function(obj) {
        return addon.encodeSync(JSON.stringify(obj));
    },
};
