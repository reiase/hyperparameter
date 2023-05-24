#ifndef _HYPERPARAMETER_H_
#define _HYPERPARAMETER_H_
#include <cstdint>
#include <string>
#include <memory>

struct Storage;

extern "C"
{
    extern Storage *hyper_create_storage();
    extern void hyper_destory_storage(Storage *);
    extern void storage_enter(Storage *);
    extern void storage_exit(Storage *);
    extern int64_t storage_hget_or_i64(Storage *, uint64_t, int64_t);
    extern double storage_hget_or_f64(Storage *, uint64_t, double);
    extern bool storage_hget_or_bool(Storage *, uint64_t, bool);
    extern char *storage_hget_or_str(Storage *, uint64_t, const char *);

    extern void storage_put_i64(Storage *, const char *, int64_t);
    extern void storage_put_f64(Storage *, const char *, double);
    extern void storage_put_bool(Storage *, const char *, bool);
    extern void storage_put_str(Storage *, const char *, const char *);
}

namespace hyperparameter
{

    struct xxh64
    {
        static constexpr uint64_t hash(const char *p, uint64_t len, uint64_t seed = 42)
        {
            return finalize((len >= 32 ? h32bytes(p, len, seed) : seed + PRIME5) + len, p + (len & ~0x1F), len & 0x1F);
        }

    private:
        static constexpr uint64_t PRIME1 = 11400714785074694791ULL;
        static constexpr uint64_t PRIME2 = 14029467366897019727ULL;
        static constexpr uint64_t PRIME3 = 1609587929392839161ULL;
        static constexpr uint64_t PRIME4 = 9650029242287828579ULL;
        static constexpr uint64_t PRIME5 = 2870177450012600261ULL;

        static constexpr uint64_t rotl(uint64_t x, int r)
        {
            return ((x << r) | (x >> (64 - r)));
        }
        static constexpr uint64_t mix1(const uint64_t h, const uint64_t prime, int rshift)
        {
            return (h ^ (h >> rshift)) * prime;
        }
        static constexpr uint64_t mix2(const uint64_t p, const uint64_t v = 0)
        {
            return rotl(v + p * PRIME2, 31) * PRIME1;
        }
        static constexpr uint64_t mix3(const uint64_t h, const uint64_t v)
        {
            return (h ^ mix2(v)) * PRIME1 + PRIME4;
        }
        static constexpr uint32_t endian32(const char *v)
        {
            return uint32_t(uint8_t(v[0])) | (uint32_t(uint8_t(v[1])) << 8) | (uint32_t(uint8_t(v[2])) << 16) | (uint32_t(uint8_t(v[3])) << 24);
        }
        static constexpr uint64_t endian64(const char *v)
        {
            return uint64_t(uint8_t(v[0])) | (uint64_t(uint8_t(v[1])) << 8) | (uint64_t(uint8_t(v[2])) << 16) | (uint64_t(uint8_t(v[3])) << 24) | (uint64_t(uint8_t(v[4])) << 32) | (uint64_t(uint8_t(v[5])) << 40) | (uint64_t(uint8_t(v[6])) << 48) | (uint64_t(uint8_t(v[7])) << 56);
        }
        static constexpr uint64_t fetch64(const char *p, const uint64_t v = 0)
        {
            return mix2(endian64(p), v);
        }
        static constexpr uint64_t fetch32(const char *p)
        {
            return uint64_t(endian32(p)) * PRIME1;
        }
        static constexpr uint64_t fetch8(const char *p)
        {
            return uint8_t(*p) * PRIME5;
        }
        static constexpr uint64_t finalize(const uint64_t h, const char *p, uint64_t len)
        {
            return (len >= 8) ? (finalize(rotl(h ^ fetch64(p), 27) * PRIME1 + PRIME4, p + 8, len - 8)) : ((len >= 4) ? (finalize(rotl(h ^ fetch32(p), 23) * PRIME2 + PRIME3, p + 4, len - 4)) : ((len > 0) ? (finalize(rotl(h ^ fetch8(p), 11) * PRIME1, p + 1, len - 1)) : (mix1(mix1(mix1(h, PRIME2, 33), PRIME3, 29), 1, 32))));
        }
        static constexpr uint64_t h32bytes(const char *p, uint64_t len, const uint64_t v1, const uint64_t v2, const uint64_t v3, const uint64_t v4)
        {
            return (len >= 32) ? h32bytes(p + 32, len - 32, fetch64(p, v1), fetch64(p + 8, v2), fetch64(p + 16, v3), fetch64(p + 24, v4)) : mix3(mix3(mix3(mix3(rotl(v1, 1) + rotl(v2, 7) + rotl(v3, 12) + rotl(v4, 18), v1), v2), v3), v4);
        }
        static constexpr uint64_t h32bytes(const char *p, uint64_t len, const uint64_t seed)
        {
            return h32bytes(p, len, seed + PRIME1 + PRIME2, seed + PRIME2, seed, seed - PRIME1);
        }
    };

    constexpr uint64_t xxhash(const char *p, int len) { return xxh64::hash(p, len, 42); }

    struct Hyperparameter
    {
        Storage *_storage;
        Hyperparameter() : _storage(hyper_create_storage()) {}
        ~Hyperparameter() { hyper_destory_storage(_storage); }

        inline void enter() { storage_enter(_storage); }
        inline void exit() { storage_exit(_storage); }

        template <typename T>
        inline T get(uint64_t key, T def);

        template <typename T>
        inline T get(const std::string &key, T def) { return get(key.c_str(), key.size(), def); }

        template <typename T>
        inline T get(const char *key, int keylen, T def) { return get(xxhash(key, keylen), def); }

        template <typename T>
        inline void put(const std::string &key, T val) { put(key.c_str(), val); }

        template <typename T>
        inline void put(const char *key, T val);
    };

    inline Hyperparameter *create() { return new Hyperparameter(); }
    inline std::shared_ptr<Hyperparameter> create_shared() { return std::make_shared<Hyperparameter>(); }

    template <>
    inline int64_t Hyperparameter::get<int64_t>(uint64_t key, int64_t def)
    {
        return storage_hget_or_i64(_storage, key, def);
    }

    template <>
    inline int32_t Hyperparameter::get<int32_t>(uint64_t key, int32_t def)
    {
        return storage_hget_or_i64(_storage, key, def);
    }

    template <>
    inline double Hyperparameter::get<double>(uint64_t key, double def)
    {
        return storage_hget_or_f64(_storage, key, def);
    }

    template <>
    inline bool Hyperparameter::get<bool>(uint64_t key, bool def)
    {
        return storage_hget_or_bool(_storage, key, def);
    }

    template <>
    inline std::string Hyperparameter::get<std::string>(uint64_t key, std::string def)
    {
        return std::string(storage_hget_or_str(_storage, key, def.c_str()));
    }

    template <>
    inline const char *Hyperparameter::get<const char *>(uint64_t key, const char *def)
    {
        return storage_hget_or_str(_storage, key, def);
    }

    template <>
    inline void Hyperparameter::put<int64_t>(const char *key, int64_t val)
    {
        return storage_put_i64(_storage, key, val);
    }

    template <>
    inline void Hyperparameter::put<int32_t>(const char *key, int32_t val)
    {
        return storage_put_i64(_storage, key, val);
    }

    template <>
    inline void Hyperparameter::put<double>(const char *key, double val)
    {
        return storage_put_f64(_storage, key, val);
    }

    template <>
    inline void Hyperparameter::put<bool>(const char *key, bool val)
    {
        return storage_put_bool(_storage, key, val);
    }

    template <>
    inline void Hyperparameter::put<const std::string &>(const char *key, const std::string &val)
    {
        return storage_put_str(_storage, key, val.c_str());
    }

    template <>
    inline void Hyperparameter::put<const char *>(const char *key, const char *val)
    {
        return storage_put_str(_storage, key, val);
    }

    inline std::shared_ptr<hyperparameter::Hyperparameter> get_hp()
    {
        static std::shared_ptr<Hyperparameter> hp;
        if (!hp)
        {
            hp = hyperparameter::create_shared();
        }
        return hp;
    }
}

#define GETHP hyperparameter::get_hp()

// Implicit create hyperparameter object
#define GETPARAM(p, default_val) \
    (GETHP->get(([]() { constexpr uint64_t x = hyperparameter::xxhash(#p,sizeof(#p)-1); return x; })(), default_val))
#define PUTPARAM(p, default_val) (GETHP->put(#p, default_val))

#endif