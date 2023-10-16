#ifndef _HYPERPARAMETER_H_
#define _HYPERPARAMETER_H_
#include <cstdint>
#include <memory>
#include <string>

struct Storage;

extern "C" {
extern Storage *param_scope_create();
extern void param_scope_destroy(Storage *);
extern void param_scope_enter(Storage *);
extern void param_scope_exit(Storage *);
extern int64_t param_scope_hget_i64(Storage *, uint64_t, int64_t);
extern double param_scope_hget_or_f64(Storage *, uint64_t, double);
extern bool param_scope_hget_or_bool(Storage *, uint64_t, bool);
extern char *param_scope_hget_or_str(Storage *, uint64_t, const char *);

extern void param_scope_put_i64(Storage *, const char *, int64_t);
extern void param_scope_put_f64(Storage *, const char *, double);
extern void param_scope_put_bool(Storage *, const char *, bool);
extern void param_scope_put_str(Storage *, const char *, const char *);
}

namespace hyperparameter {

struct xxh64 {
  static constexpr uint64_t hash(const char *p, uint64_t len,
                                 uint64_t seed = 42) {
    return finalize((len >= 32 ? h32bytes(p, len, seed) : seed + PRIME5) + len,
                    p + (len & ~0x1F), len & 0x1F);
  }

private:
  static constexpr uint64_t PRIME1 = 11400714785074694791ULL;
  static constexpr uint64_t PRIME2 = 14029467366897019727ULL;
  static constexpr uint64_t PRIME3 = 1609587929392839161ULL;
  static constexpr uint64_t PRIME4 = 9650029242287828579ULL;
  static constexpr uint64_t PRIME5 = 2870177450012600261ULL;

  static constexpr uint64_t rotl(uint64_t x, int r) {
    return ((x << r) | (x >> (64 - r)));
  }
  static constexpr uint64_t mix1(const uint64_t h, const uint64_t prime,
                                 int rshift) {
    return (h ^ (h >> rshift)) * prime;
  }
  static constexpr uint64_t mix2(const uint64_t p, const uint64_t v = 0) {
    return rotl(v + p * PRIME2, 31) * PRIME1;
  }
  static constexpr uint64_t mix3(const uint64_t h, const uint64_t v) {
    return (h ^ mix2(v)) * PRIME1 + PRIME4;
  }
  static constexpr uint32_t endian32(const char *v) {
    return uint32_t(uint8_t(v[0])) | (uint32_t(uint8_t(v[1])) << 8) |
           (uint32_t(uint8_t(v[2])) << 16) | (uint32_t(uint8_t(v[3])) << 24);
  }
  static constexpr uint64_t endian64(const char *v) {
    return uint64_t(uint8_t(v[0])) | (uint64_t(uint8_t(v[1])) << 8) |
           (uint64_t(uint8_t(v[2])) << 16) | (uint64_t(uint8_t(v[3])) << 24) |
           (uint64_t(uint8_t(v[4])) << 32) | (uint64_t(uint8_t(v[5])) << 40) |
           (uint64_t(uint8_t(v[6])) << 48) | (uint64_t(uint8_t(v[7])) << 56);
  }
  static constexpr uint64_t fetch64(const char *p, const uint64_t v = 0) {
    return mix2(endian64(p), v);
  }
  static constexpr uint64_t fetch32(const char *p) {
    return uint64_t(endian32(p)) * PRIME1;
  }
  static constexpr uint64_t fetch8(const char *p) {
    return uint8_t(*p) * PRIME5;
  }
  static constexpr uint64_t finalize(const uint64_t h, const char *p,
                                     uint64_t len) {
    return (len >= 8)
               ? (finalize(rotl(h ^ fetch64(p), 27) * PRIME1 + PRIME4, p + 8,
                           len - 8))
               : ((len >= 4)
                      ? (finalize(rotl(h ^ fetch32(p), 23) * PRIME2 + PRIME3,
                                  p + 4, len - 4))
                      : ((len > 0)
                             ? (finalize(rotl(h ^ fetch8(p), 11) * PRIME1,
                                         p + 1, len - 1))
                             : (mix1(mix1(mix1(h, PRIME2, 33), PRIME3, 29), 1,
                                     32))));
  }
  static constexpr uint64_t h32bytes(const char *p, uint64_t len,
                                     const uint64_t v1, const uint64_t v2,
                                     const uint64_t v3, const uint64_t v4) {
    return (len >= 32)
               ? h32bytes(p + 32, len - 32, fetch64(p, v1), fetch64(p + 8, v2),
                          fetch64(p + 16, v3), fetch64(p + 24, v4))
               : mix3(mix3(mix3(mix3(rotl(v1, 1) + rotl(v2, 7) + rotl(v3, 12) +
                                         rotl(v4, 18),
                                     v1),
                                v2),
                           v3),
                      v4);
  }
  static constexpr uint64_t h32bytes(const char *p, uint64_t len,
                                     const uint64_t seed) {
    return h32bytes(p, len, seed + PRIME1 + PRIME2, seed + PRIME2, seed,
                    seed - PRIME1);
  }
};

constexpr uint64_t xxhash(const char *p, int len) {
  return xxh64::hash(p, len, 42);
}

struct Hyperparameter {
  Storage *_storage;
  bool _is_entered;
  Hyperparameter() : _storage(param_scope_create()), _is_entered(false) {}
  ~Hyperparameter() {
    if (_is_entered)
      param_scope_exit(_storage);
    param_scope_destroy(_storage);
  }

  inline Hyperparameter *enter() {
    param_scope_enter(_storage);
    _is_entered = true;
    return this;
  }
  inline Hyperparameter *exit() {
    param_scope_exit(_storage);
    return this;
  }

  inline Hyperparameter *self() { return this; }

  template <typename T> inline T get(uint64_t key, T def);

  template <typename T> inline T get(const std::string &key, T def) {
    return get(key.c_str(), key.size(), def);
  }

  template <typename T> inline T get(const char *key, int keylen, T def) {
    return get(xxhash(key, keylen), def);
  }

  template <typename T>
  inline Hyperparameter *put(const std::string &key, T val) {
    put(key.c_str(), val);
    return this;
  }

  template <typename T> inline Hyperparameter *put(const char *key, T val);
};

inline std::shared_ptr<Hyperparameter> create_shared() {
  return std::make_shared<Hyperparameter>();
}
inline Hyperparameter *create() { return new Hyperparameter(); }

template <>
inline int64_t Hyperparameter::get<int64_t>(uint64_t key, int64_t def) {
  return param_scope_hget_i64(_storage, key, def);
}

template <>
inline int32_t Hyperparameter::get<int32_t>(uint64_t key, int32_t def) {
  return param_scope_hget_i64(_storage, key, def);
}

template <>
inline double Hyperparameter::get<double>(uint64_t key, double def) {
  return param_scope_hget_or_f64(_storage, key, def);
}

template <> inline bool Hyperparameter::get<bool>(uint64_t key, bool def) {
  return param_scope_hget_or_bool(_storage, key, def);
}

template <>
inline std::string Hyperparameter::get<std::string>(uint64_t key,
                                                    std::string def) {
  return std::string(param_scope_hget_or_str(_storage, key, def.c_str()));
}

template <>
inline const char *Hyperparameter::get<const char *>(uint64_t key,
                                                     const char *def) {
  return param_scope_hget_or_str(_storage, key, def);
}

template <>
inline Hyperparameter *Hyperparameter::put<int64_t>(const char *key,
                                                    int64_t val) {
  param_scope_put_i64(_storage, key, val);
  return this;
}

template <>
inline Hyperparameter *Hyperparameter::put<int32_t>(const char *key,
                                                    int32_t val) {
  param_scope_put_i64(_storage, key, val);
  return this;
}

template <>
inline Hyperparameter *Hyperparameter::put<double>(const char *key,
                                                   double val) {
  param_scope_put_f64(_storage, key, val);
  return this;
}

template <>
inline Hyperparameter *Hyperparameter::put<bool>(const char *key, bool val) {
  param_scope_put_bool(_storage, key, val);
  return this;
}

template <>
inline Hyperparameter *
Hyperparameter::put<const std::string &>(const char *key,
                                         const std::string &val) {
  param_scope_put_str(_storage, key, val.c_str());
  return this;
}

template <>
inline Hyperparameter *Hyperparameter::put<const char *>(const char *key,
                                                         const char *val) {
  param_scope_put_str(_storage, key, val);
  return this;
}
} // namespace hyperparameter

// Implicit create hyperparameter object
#define GET_PARAM(p, default_val)                                              \
  (hyperparameter::Hyperparameter().get(                                       \
      ([]() {                                                                  \
        constexpr uint64_t x = hyperparameter::xxhash(#p, sizeof(#p) - 1);     \
        return x;                                                              \
      })(),                                                                    \
      (default_val)))
#define GETPARAM(p, def) GET_PARAM(p, (def))

#define KEYHASH(p)                                                             \
  (([]() {                                                                     \
    constexpr uint64_t x = hyperparameter::xxhash(#p, sizeof(#p) - 1);         \
    return x;                                                                  \
  })())

#define PP_ARG_X(_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, a, b, c, d, e, f, g,  \
                 h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, A,   \
                 B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U,   \
                 V, W, X, Y, Z, XX, ...)                                       \
  XX

#define PP_ARG_N(...)                                                          \
  PP_ARG_X("ignored", ##__VA_ARGS__, Z, Y, X, W, V, U, T, S, R, Q, P, O, N, M, \
           L, K, J, I, H, G, F, E, D, C, B, A, 35, 34, 33, 32, 31, 30, 29, 28, \
           27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, \
           10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)

#define PP_VA_NAME(prefix, ...) PP_CAT2(prefix, PP_ARG_N(__VA_ARGS__))
#define PP_CAT2(a, b) PP_CAT2_1(a, b)
#define PP_CAT2_1(a, b) a##b

#define WITH_PARAMS_0(impl) (impl->enter())
#define WITH_PARAMS_2(impl, k, v) WITH_PARAMS_0(((impl)->put(#k, v)))
#define WITH_PARAMS_4(impl, k, v, ...)                                         \
  WITH_PARAMS_2((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_6(impl, k, v, ...)                                         \
  WITH_PARAMS_4((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_8(impl, k, v, ...)                                         \
  WITH_PARAMS_6((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_10(impl, k, v, ...)                                        \
  WITH_PARAMS_8((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_12(impl, k, v, ...)                                        \
  WITH_PARAMS_10((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_14(impl, k, v, ...)                                        \
  WITH_PARAMS_12((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_16(impl, k, v, ...)                                        \
  WITH_PARAMS_14((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_18(impl, k, v, ...)                                        \
  WITH_PARAMS_16((impl)->put(#k, v), ##__VA_ARGS__)
#define WITH_PARAMS_20(impl, k, v, ...)                                        \
  WITH_PARAMS_18((impl)->put(#k, v), ##__VA_ARGS__)

#define WITH_PARAMS_HELPER(...)                                                \
  PP_VA_NAME(WITH_PARAMS_, __VA_ARGS__)(hyperparameter::create(), __VA_ARGS__)

#define WITH_PARAMS(...)                                                       \
  std::unique_ptr<hyperparameter::Hyperparameter>(                             \
      WITH_PARAMS_HELPER(__VA_ARGS__))

#endif
