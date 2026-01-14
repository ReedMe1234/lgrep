#include "lib.hpp"
#include <algorithm>
#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <initializer_list>
#include <iterator>
#include <memory>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
#if __cplusplus >= 201703L
#include <string_view>
#endif
#if __cplusplus >= 202002L
#include <ranges>
#endif

#ifdef __GNUC__
#pragma GCC diagnostic ignored "-Wmissing-declarations"
#pragma GCC diagnostic ignored "-Wshadow"
#ifdef __clang__
#pragma clang diagnostic ignored "-Wdollar-in-identifier-extension"
#endif // __clang__
#endif // __GNUC__

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

#ifndef CXXBRIDGE1_PANIC
#define CXXBRIDGE1_PANIC
template <typename Exception>
void panic [[noreturn]] (const char *msg);
#endif // CXXBRIDGE1_PANIC

namespace {
template <typename T>
class impl;
} // namespace

class String;
class Opaque;

template <typename T>
::std::size_t size_of();
template <typename T>
::std::size_t align_of();

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const String &) noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, std::size_t);

  Str &operator=(const Str &) & noexcept = default;

  explicit operator std::string() const;
#if __cplusplus >= 201703L
  explicit operator std::string_view() const;
#endif

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const Str &) const noexcept;
  bool operator!=(const Str &) const noexcept;
  bool operator<(const Str &) const noexcept;
  bool operator<=(const Str &) const noexcept;
  bool operator>(const Str &) const noexcept;
  bool operator>=(const Str &) const noexcept;

  void swap(Str &) noexcept;

private:
  class uninit;
  Str(uninit) noexcept;
  friend impl<Str>;

  std::array<std::uintptr_t, 2> repr;
};
#endif // CXXBRIDGE1_RUST_STR

#ifndef CXXBRIDGE1_RUST_SLICE
#define CXXBRIDGE1_RUST_SLICE
namespace detail {
template <bool>
struct copy_assignable_if {};

template <>
struct copy_assignable_if<false> {
  copy_assignable_if() noexcept = default;
  copy_assignable_if(const copy_assignable_if &) noexcept = default;
  copy_assignable_if &operator=(const copy_assignable_if &) & noexcept = delete;
  copy_assignable_if &operator=(copy_assignable_if &&) & noexcept = default;
};
} // namespace detail

template <typename T>
class Slice final
    : private detail::copy_assignable_if<std::is_const<T>::value> {
public:
  using value_type = T;

  Slice() noexcept;
  Slice(T *, std::size_t count) noexcept;

  template <typename C>
  explicit Slice(C &c) : Slice(c.data(), c.size()) {}

  Slice &operator=(const Slice<T> &) & noexcept = default;
  Slice &operator=(Slice<T> &&) & noexcept = default;

  T *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  T &operator[](std::size_t n) const noexcept;
  T &at(std::size_t n) const;
  T &front() const noexcept;
  T &back() const noexcept;

  Slice(const Slice<T> &) noexcept = default;
  ~Slice() noexcept = default;

  class iterator;
  iterator begin() const noexcept;
  iterator end() const noexcept;

  void swap(Slice &) noexcept;

private:
  class uninit;
  Slice(uninit) noexcept;
  friend impl<Slice>;
  friend void sliceInit(void *, const void *, std::size_t) noexcept;
  friend void *slicePtr(const void *) noexcept;
  friend std::size_t sliceLen(const void *) noexcept;

  std::array<std::uintptr_t, 2> repr;
};

#ifdef __cpp_deduction_guides
template <typename C>
explicit Slice(C &c)
    -> Slice<std::remove_reference_t<decltype(*std::declval<C>().data())>>;
#endif // __cpp_deduction_guides

template <typename T>
class Slice<T>::iterator final {
public:
#if __cplusplus >= 202002L
  using iterator_category = std::contiguous_iterator_tag;
#else
  using iterator_category = std::random_access_iterator_tag;
#endif
  using value_type = T;
  using difference_type = std::ptrdiff_t;
  using pointer = typename std::add_pointer<T>::type;
  using reference = typename std::add_lvalue_reference<T>::type;

  reference operator*() const noexcept;
  pointer operator->() const noexcept;
  reference operator[](difference_type) const noexcept;

  iterator &operator++() noexcept;
  iterator operator++(int) noexcept;
  iterator &operator--() noexcept;
  iterator operator--(int) noexcept;

  iterator &operator+=(difference_type) noexcept;
  iterator &operator-=(difference_type) noexcept;
  iterator operator+(difference_type) const noexcept;
  friend inline iterator operator+(difference_type lhs, iterator rhs) noexcept {
    return rhs + lhs;
  }
  iterator operator-(difference_type) const noexcept;
  difference_type operator-(const iterator &) const noexcept;

  bool operator==(const iterator &) const noexcept;
  bool operator!=(const iterator &) const noexcept;
  bool operator<(const iterator &) const noexcept;
  bool operator<=(const iterator &) const noexcept;
  bool operator>(const iterator &) const noexcept;
  bool operator>=(const iterator &) const noexcept;

private:
  friend class Slice;
  void *pos;
  std::size_t stride;
};

#if __cplusplus >= 202002L
static_assert(std::ranges::contiguous_range<rust::Slice<const uint8_t>>);
static_assert(std::contiguous_iterator<rust::Slice<const uint8_t>::iterator>);
#endif

template <typename T>
Slice<T>::Slice() noexcept {
  sliceInit(this, reinterpret_cast<void *>(align_of<T>()), 0);
}

template <typename T>
Slice<T>::Slice(T *s, std::size_t count) noexcept {
  assert(s != nullptr || count == 0);
  sliceInit(this,
            s == nullptr && count == 0
                ? reinterpret_cast<void *>(align_of<T>())
                : const_cast<typename std::remove_const<T>::type *>(s),
            count);
}

template <typename T>
T *Slice<T>::data() const noexcept {
  return reinterpret_cast<T *>(slicePtr(this));
}

template <typename T>
std::size_t Slice<T>::size() const noexcept {
  return sliceLen(this);
}

template <typename T>
std::size_t Slice<T>::length() const noexcept {
  return this->size();
}

template <typename T>
bool Slice<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T &Slice<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto ptr = static_cast<char *>(slicePtr(this)) + size_of<T>() * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
T &Slice<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Slice index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Slice<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Slice<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
typename Slice<T>::iterator::reference
Slice<T>::iterator::operator*() const noexcept {
  return *static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::pointer
Slice<T>::iterator::operator->() const noexcept {
  return static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::reference Slice<T>::iterator::operator[](
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ptr = static_cast<char *>(this->pos) + this->stride * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator++() noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator++(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator--() noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator--(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator+=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator-=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator+(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) + this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator-(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) - this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator::difference_type
Slice<T>::iterator::operator-(const iterator &other) const noexcept {
  auto diff = std::distance(static_cast<char *>(other.pos),
                            static_cast<char *>(this->pos));
  return diff / static_cast<typename Slice<T>::iterator::difference_type>(
                    this->stride);
}

template <typename T>
bool Slice<T>::iterator::operator==(const iterator &other) const noexcept {
  return this->pos == other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator!=(const iterator &other) const noexcept {
  return this->pos != other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<(const iterator &other) const noexcept {
  return this->pos < other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<=(const iterator &other) const noexcept {
  return this->pos <= other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>(const iterator &other) const noexcept {
  return this->pos > other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>=(const iterator &other) const noexcept {
  return this->pos >= other.pos;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::begin() const noexcept {
  iterator it;
  it.pos = slicePtr(this);
  it.stride = size_of<T>();
  return it;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::end() const noexcept {
  iterator it = this->begin();
  it.pos = static_cast<char *>(it.pos) + it.stride * this->size();
  return it;
}

template <typename T>
void Slice<T>::swap(Slice &rhs) noexcept {
  std::swap(*this, rhs);
}
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BITCOPY_T
#define CXXBRIDGE1_RUST_BITCOPY_T
struct unsafe_bitcopy_t final {
  explicit unsafe_bitcopy_t() = default;
};
#endif // CXXBRIDGE1_RUST_BITCOPY_T

#ifndef CXXBRIDGE1_RUST_VEC
#define CXXBRIDGE1_RUST_VEC
template <typename T>
class Vec final {
public:
  using value_type = T;

  Vec() noexcept;
  Vec(std::initializer_list<T>);
  Vec(const Vec &);
  Vec(Vec &&) noexcept;
  ~Vec() noexcept;

  Vec &operator=(Vec &&) & noexcept;
  Vec &operator=(const Vec &) &;

  std::size_t size() const noexcept;
  bool empty() const noexcept;
  const T *data() const noexcept;
  T *data() noexcept;
  std::size_t capacity() const noexcept;

  const T &operator[](std::size_t n) const noexcept;
  const T &at(std::size_t n) const;
  const T &front() const noexcept;
  const T &back() const noexcept;

  T &operator[](std::size_t n) noexcept;
  T &at(std::size_t n);
  T &front() noexcept;
  T &back() noexcept;

  void reserve(std::size_t new_cap);
  void push_back(const T &value);
  void push_back(T &&value);
  template <typename... Args>
  void emplace_back(Args &&...args);
  void truncate(std::size_t len);
  void clear();

  using iterator = typename Slice<T>::iterator;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = typename Slice<const T>::iterator;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  void swap(Vec &) noexcept;

  Vec(unsafe_bitcopy_t, const Vec &) noexcept;

private:
  void reserve_total(std::size_t new_cap) noexcept;
  void set_len(std::size_t len) noexcept;
  void drop() noexcept;

  friend void swap(Vec &lhs, Vec &rhs) noexcept { lhs.swap(rhs); }

  std::array<std::uintptr_t, 3> repr;
};

template <typename T>
Vec<T>::Vec(std::initializer_list<T> init) : Vec{} {
  this->reserve_total(init.size());
  std::move(init.begin(), init.end(), std::back_inserter(*this));
}

template <typename T>
Vec<T>::Vec(const Vec &other) : Vec() {
  this->reserve_total(other.size());
  std::copy(other.begin(), other.end(), std::back_inserter(*this));
}

template <typename T>
Vec<T>::Vec(Vec &&other) noexcept : repr(other.repr) {
  new (&other) Vec();
}

template <typename T>
Vec<T>::~Vec() noexcept {
  this->drop();
}

template <typename T>
Vec<T> &Vec<T>::operator=(Vec &&other) & noexcept {
  this->drop();
  this->repr = other.repr;
  new (&other) Vec();
  return *this;
}

template <typename T>
Vec<T> &Vec<T>::operator=(const Vec &other) & {
  if (this != &other) {
    this->drop();
    new (this) Vec(other);
  }
  return *this;
}

template <typename T>
bool Vec<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T *Vec<T>::data() noexcept {
  return const_cast<T *>(const_cast<const Vec<T> *>(this)->data());
}

template <typename T>
const T &Vec<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto data = reinterpret_cast<const char *>(this->data());
  return *reinterpret_cast<const T *>(data + n * size_of<T>());
}

template <typename T>
const T &Vec<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Vec index out of range");
  }
  return (*this)[n];
}

template <typename T>
const T &Vec<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
const T &Vec<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
T &Vec<T>::operator[](std::size_t n) noexcept {
  assert(n < this->size());
  auto data = reinterpret_cast<char *>(this->data());
  return *reinterpret_cast<T *>(data + n * size_of<T>());
}

template <typename T>
T &Vec<T>::at(std::size_t n) {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Vec index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Vec<T>::front() noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Vec<T>::back() noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
void Vec<T>::reserve(std::size_t new_cap) {
  this->reserve_total(new_cap);
}

template <typename T>
void Vec<T>::push_back(const T &value) {
  this->emplace_back(value);
}

template <typename T>
void Vec<T>::push_back(T &&value) {
  this->emplace_back(std::move(value));
}

template <typename T>
template <typename... Args>
void Vec<T>::emplace_back(Args &&...args) {
  auto size = this->size();
  this->reserve_total(size + 1);
  ::new (reinterpret_cast<T *>(reinterpret_cast<char *>(this->data()) +
                               size * size_of<T>()))
      T(std::forward<Args>(args)...);
  this->set_len(size + 1);
}

template <typename T>
void Vec<T>::clear() {
  this->truncate(0);
}

template <typename T>
typename Vec<T>::iterator Vec<T>::begin() noexcept {
  return Slice<T>(this->data(), this->size()).begin();
}

template <typename T>
typename Vec<T>::iterator Vec<T>::end() noexcept {
  return Slice<T>(this->data(), this->size()).end();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::begin() const noexcept {
  return this->cbegin();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::end() const noexcept {
  return this->cend();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::cbegin() const noexcept {
  return Slice<const T>(this->data(), this->size()).begin();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::cend() const noexcept {
  return Slice<const T>(this->data(), this->size()).end();
}

template <typename T>
void Vec<T>::swap(Vec &rhs) noexcept {
  using std::swap;
  swap(this->repr, rhs.repr);
}

template <typename T>
Vec<T>::Vec(unsafe_bitcopy_t, const Vec &bits) noexcept : repr(bits.repr) {}
#endif // CXXBRIDGE1_RUST_VEC

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

#ifndef CXXBRIDGE1_LAYOUT
#define CXXBRIDGE1_LAYOUT
class layout {
  template <typename T>
  friend std::size_t size_of();
  template <typename T>
  friend std::size_t align_of();
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return T::layout::size();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return sizeof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      size_of() {
    return do_size_of<T>();
  }
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return T::layout::align();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return alignof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      align_of() {
    return do_align_of<T>();
  }
};

template <typename T>
std::size_t size_of() {
  return layout::size_of<T>();
}

template <typename T>
std::size_t align_of() {
  return layout::align_of<T>();
}
#endif // CXXBRIDGE1_LAYOUT

namespace repr {
struct PtrLen final {
  void *ptr;
  ::std::size_t len;
};
} // namespace repr

namespace detail {
class Fail final {
  ::rust::repr::PtrLen &throw$;
public:
  Fail(::rust::repr::PtrLen &throw$) noexcept : throw$(throw$) {}
  void operator()(char const *) noexcept;
  void operator()(std::string const &) noexcept;
};
} // namespace detail

namespace {
template <bool> struct deleter_if {
  template <typename T> void operator()(T *) {}
};
template <> struct deleter_if<true> {
  template <typename T> void operator()(T *ptr) { ptr->~T(); }
};
} // namespace
} // namespace cxxbridge1

namespace behavior {
class missing {};
missing trycatch(...);

template <typename Try, typename Fail>
static typename ::std::enable_if<::std::is_same<
    decltype(trycatch(::std::declval<Try>(), ::std::declval<Fail>())),
    missing>::value>::type
trycatch(Try &&func, Fail &&fail) noexcept try {
  func();
} catch (::std::exception const &e) {
  fail(e.what());
}
} // namespace behavior
} // namespace rust

#if __cplusplus >= 201402L
#define CXX_DEFAULT_VALUE(value) = value
#else
#define CXX_DEFAULT_VALUE(value)
#endif

enum class MetricKind : ::std::int32_t;
enum class ScalarKind : ::std::int32_t;
struct Matches;
struct IndexOptions;
using NativeIndex = ::NativeIndex;

#ifndef CXXBRIDGE1_ENUM_MetricKind
#define CXXBRIDGE1_ENUM_MetricKind
// The metric kind used to differentiate built-in distance functions.
enum class MetricKind : ::std::int32_t {
  Unknown = 0,
  // The Inner Product metric, defined as `IP = 1 - sum(a[i] * b[i])`.
  IP = 1,
  // The squared Euclidean Distance metric, defined as `L2 = sum((a[i] - b[i])^2)`.
  L2sq = 2,
  // The Cosine Similarity metric, defined as `Cos = 1 - sum(a[i] * b[i]) / (sqrt(sum(a[i]^2) * sqrt(sum(b[i]^2)))`.
  Cos = 3,
  // The Pearson Correlation metric.
  Pearson = 4,
  // The Haversine (Great Circle) Distance metric.
  Haversine = 5,
  // The Jensen Shannon Divergence metric.
  Divergence = 6,
  // The bit-level Hamming Distance metric, defined as the number of differing bits.
  Hamming = 7,
  // The bit-level Tanimoto (Jaccard) metric, defined as the number of intersecting bits divided by the number of union bits.
  Tanimoto = 8,
  // The bit-level Sorensen metric.
  Sorensen = 9,
};
#endif // CXXBRIDGE1_ENUM_MetricKind

#ifndef CXXBRIDGE1_ENUM_ScalarKind
#define CXXBRIDGE1_ENUM_ScalarKind
// The scalar kind used to differentiate built-in vector element types.
enum class ScalarKind : ::std::int32_t {
  Unknown = 0,
  // 64-bit double-precision IEEE 754 floating-point number.
  F64 = 1,
  // 32-bit single-precision IEEE 754 floating-point number.
  F32 = 2,
  // 16-bit half-precision IEEE 754 floating-point number (different from `bf16`).
  F16 = 3,
  // 16-bit brain floating-point number.
  BF16 = 4,
  // 8-bit signed integer.
  I8 = 5,
  // 1-bit binary value, packed 8 per byte.
  B1 = 6,
};
#endif // CXXBRIDGE1_ENUM_ScalarKind

#ifndef CXXBRIDGE1_STRUCT_Matches
#define CXXBRIDGE1_STRUCT_Matches
// The resulting matches from a search operation.
// It contains the keys and distances of the closest vectors.
struct Matches final {
  ::rust::Vec<::std::uint64_t> keys;
  ::rust::Vec<float> distances;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_Matches

#ifndef CXXBRIDGE1_STRUCT_IndexOptions
#define CXXBRIDGE1_STRUCT_IndexOptions
// The index options used to configure the dense index during creation.
// It contains the number of dimensions, the metric kind, the scalar kind,
// the connectivity, the expansion values, and the multi-flag.
struct IndexOptions final {
  ::std::size_t dimensions CXX_DEFAULT_VALUE(0);
  ::MetricKind metric;
  ::ScalarKind quantization;
  ::std::size_t connectivity CXX_DEFAULT_VALUE(0);
  ::std::size_t expansion_add CXX_DEFAULT_VALUE(0);
  ::std::size_t expansion_search CXX_DEFAULT_VALUE(0);
  bool multi CXX_DEFAULT_VALUE(false);

  bool operator==(IndexOptions const &) const noexcept;
  bool operator!=(IndexOptions const &) const noexcept;
  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_IndexOptions

extern "C" {
bool cxxbridge1$192$IndexOptions$operator$eq(IndexOptions const &, IndexOptions const &) noexcept;
bool cxxbridge1$192$IndexOptions$operator$ne(IndexOptions const &, IndexOptions const &) noexcept;

::std::size_t cxxbridge1$192$NativeIndex$expansion_add(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*expansion_add$)() const = &::NativeIndex::expansion_add;
  return (self.*expansion_add$)();
}

::std::size_t cxxbridge1$192$NativeIndex$expansion_search(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*expansion_search$)() const = &::NativeIndex::expansion_search;
  return (self.*expansion_search$)();
}

void cxxbridge1$192$NativeIndex$change_expansion_add(::NativeIndex const &self, ::std::size_t n) noexcept {
  void (::NativeIndex::*change_expansion_add$)(::std::size_t) const = &::NativeIndex::change_expansion_add;
  (self.*change_expansion_add$)(n);
}

void cxxbridge1$192$NativeIndex$change_expansion_search(::NativeIndex const &self, ::std::size_t n) noexcept {
  void (::NativeIndex::*change_expansion_search$)(::std::size_t) const = &::NativeIndex::change_expansion_search;
  (self.*change_expansion_search$)(n);
}

void cxxbridge1$192$NativeIndex$change_metric_kind(::NativeIndex const &self, ::MetricKind metric) noexcept {
  void (::NativeIndex::*change_metric_kind$)(::MetricKind) const = &::NativeIndex::change_metric_kind;
  (self.*change_metric_kind$)(metric);
}

void cxxbridge1$192$NativeIndex$change_metric(::NativeIndex const &self, ::std::size_t metric, ::std::size_t metric_state) noexcept {
  void (::NativeIndex::*change_metric$)(::std::size_t, ::std::size_t) const = &::NativeIndex::change_metric;
  (self.*change_metric$)(metric, metric_state);
}

::rust::repr::PtrLen cxxbridge1$192$new_native_index(::IndexOptions const &options, ::NativeIndex **return$) noexcept {
  ::std::unique_ptr<::NativeIndex> (*new_native_index$)(::IndexOptions const &) = ::new_native_index;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::NativeIndex *(new_native_index$(options).release());
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$reserve(::NativeIndex const &self, ::std::size_t capacity) noexcept {
  void (::NativeIndex::*reserve$)(::std::size_t) const = &::NativeIndex::reserve;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*reserve$)(capacity);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$reserve_capacity_and_threads(::NativeIndex const &self, ::std::size_t capacity, ::std::size_t threads) noexcept {
  void (::NativeIndex::*reserve_capacity_and_threads$)(::std::size_t, ::std::size_t) const = &::NativeIndex::reserve_capacity_and_threads;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*reserve_capacity_and_threads$)(capacity, threads);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::std::size_t cxxbridge1$192$NativeIndex$dimensions(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*dimensions$)() const = &::NativeIndex::dimensions;
  return (self.*dimensions$)();
}

::std::size_t cxxbridge1$192$NativeIndex$connectivity(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*connectivity$)() const = &::NativeIndex::connectivity;
  return (self.*connectivity$)();
}

::std::size_t cxxbridge1$192$NativeIndex$size(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*size$)() const = &::NativeIndex::size;
  return (self.*size$)();
}

::std::size_t cxxbridge1$192$NativeIndex$capacity(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*capacity$)() const = &::NativeIndex::capacity;
  return (self.*capacity$)();
}

::std::size_t cxxbridge1$192$NativeIndex$serialized_length(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*serialized_length$)() const = &::NativeIndex::serialized_length;
  return (self.*serialized_length$)();
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$add_b1x8(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<::std::uint8_t const> vector) noexcept {
  void (::NativeIndex::*add_b1x8$)(::std::uint64_t, ::rust::Slice<::std::uint8_t const>) const = &::NativeIndex::add_b1x8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*add_b1x8$)(key, vector);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$add_i8(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<::std::int8_t const> vector) noexcept {
  void (::NativeIndex::*add_i8$)(::std::uint64_t, ::rust::Slice<::std::int8_t const>) const = &::NativeIndex::add_i8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*add_i8$)(key, vector);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$add_f16(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<::std::int16_t const> vector) noexcept {
  void (::NativeIndex::*add_f16$)(::std::uint64_t, ::rust::Slice<::std::int16_t const>) const = &::NativeIndex::add_f16;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*add_f16$)(key, vector);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$add_f32(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<float const> vector) noexcept {
  void (::NativeIndex::*add_f32$)(::std::uint64_t, ::rust::Slice<float const>) const = &::NativeIndex::add_f32;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*add_f32$)(key, vector);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$add_f64(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<double const> vector) noexcept {
  void (::NativeIndex::*add_f64$)(::std::uint64_t, ::rust::Slice<double const>) const = &::NativeIndex::add_f64;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*add_f64$)(key, vector);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$search_b1x8(::NativeIndex const &self, ::rust::Slice<::std::uint8_t const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*search_b1x8$)(::rust::Slice<::std::uint8_t const>, ::std::size_t) const = &::NativeIndex::search_b1x8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*search_b1x8$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$search_i8(::NativeIndex const &self, ::rust::Slice<::std::int8_t const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*search_i8$)(::rust::Slice<::std::int8_t const>, ::std::size_t) const = &::NativeIndex::search_i8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*search_i8$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$search_f16(::NativeIndex const &self, ::rust::Slice<::std::int16_t const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*search_f16$)(::rust::Slice<::std::int16_t const>, ::std::size_t) const = &::NativeIndex::search_f16;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*search_f16$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$search_f32(::NativeIndex const &self, ::rust::Slice<float const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*search_f32$)(::rust::Slice<float const>, ::std::size_t) const = &::NativeIndex::search_f32;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*search_f32$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$search_f64(::NativeIndex const &self, ::rust::Slice<double const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*search_f64$)(::rust::Slice<double const>, ::std::size_t) const = &::NativeIndex::search_f64;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*search_f64$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$exact_search_b1x8(::NativeIndex const &self, ::rust::Slice<::std::uint8_t const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*exact_search_b1x8$)(::rust::Slice<::std::uint8_t const>, ::std::size_t) const = &::NativeIndex::exact_search_b1x8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*exact_search_b1x8$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$exact_search_i8(::NativeIndex const &self, ::rust::Slice<::std::int8_t const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*exact_search_i8$)(::rust::Slice<::std::int8_t const>, ::std::size_t) const = &::NativeIndex::exact_search_i8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*exact_search_i8$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$exact_search_f16(::NativeIndex const &self, ::rust::Slice<::std::int16_t const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*exact_search_f16$)(::rust::Slice<::std::int16_t const>, ::std::size_t) const = &::NativeIndex::exact_search_f16;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*exact_search_f16$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$exact_search_f32(::NativeIndex const &self, ::rust::Slice<float const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*exact_search_f32$)(::rust::Slice<float const>, ::std::size_t) const = &::NativeIndex::exact_search_f32;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*exact_search_f32$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$exact_search_f64(::NativeIndex const &self, ::rust::Slice<double const> query, ::std::size_t count, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*exact_search_f64$)(::rust::Slice<double const>, ::std::size_t) const = &::NativeIndex::exact_search_f64;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*exact_search_f64$)(query, count));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$filtered_search_b1x8(::NativeIndex const &self, ::rust::Slice<::std::uint8_t const> query, ::std::size_t count, ::std::size_t filter, ::std::size_t filter_state, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*filtered_search_b1x8$)(::rust::Slice<::std::uint8_t const>, ::std::size_t, ::std::size_t, ::std::size_t) const = &::NativeIndex::filtered_search_b1x8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*filtered_search_b1x8$)(query, count, filter, filter_state));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$filtered_search_i8(::NativeIndex const &self, ::rust::Slice<::std::int8_t const> query, ::std::size_t count, ::std::size_t filter, ::std::size_t filter_state, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*filtered_search_i8$)(::rust::Slice<::std::int8_t const>, ::std::size_t, ::std::size_t, ::std::size_t) const = &::NativeIndex::filtered_search_i8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*filtered_search_i8$)(query, count, filter, filter_state));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$filtered_search_f16(::NativeIndex const &self, ::rust::Slice<::std::int16_t const> query, ::std::size_t count, ::std::size_t filter, ::std::size_t filter_state, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*filtered_search_f16$)(::rust::Slice<::std::int16_t const>, ::std::size_t, ::std::size_t, ::std::size_t) const = &::NativeIndex::filtered_search_f16;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*filtered_search_f16$)(query, count, filter, filter_state));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$filtered_search_f32(::NativeIndex const &self, ::rust::Slice<float const> query, ::std::size_t count, ::std::size_t filter, ::std::size_t filter_state, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*filtered_search_f32$)(::rust::Slice<float const>, ::std::size_t, ::std::size_t, ::std::size_t) const = &::NativeIndex::filtered_search_f32;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*filtered_search_f32$)(query, count, filter, filter_state));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$filtered_search_f64(::NativeIndex const &self, ::rust::Slice<double const> query, ::std::size_t count, ::std::size_t filter, ::std::size_t filter_state, ::Matches *return$) noexcept {
  ::Matches (::NativeIndex::*filtered_search_f64$)(::rust::Slice<double const>, ::std::size_t, ::std::size_t, ::std::size_t) const = &::NativeIndex::filtered_search_f64;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::Matches((self.*filtered_search_f64$)(query, count, filter, filter_state));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$get_b1x8(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<::std::uint8_t > buffer, ::std::size_t *return$) noexcept {
  ::std::size_t (::NativeIndex::*get_b1x8$)(::std::uint64_t, ::rust::Slice<::std::uint8_t >) const = &::NativeIndex::get_b1x8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::size_t((self.*get_b1x8$)(key, buffer));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$get_i8(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<::std::int8_t > buffer, ::std::size_t *return$) noexcept {
  ::std::size_t (::NativeIndex::*get_i8$)(::std::uint64_t, ::rust::Slice<::std::int8_t >) const = &::NativeIndex::get_i8;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::size_t((self.*get_i8$)(key, buffer));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$get_f16(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<::std::int16_t > buffer, ::std::size_t *return$) noexcept {
  ::std::size_t (::NativeIndex::*get_f16$)(::std::uint64_t, ::rust::Slice<::std::int16_t >) const = &::NativeIndex::get_f16;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::size_t((self.*get_f16$)(key, buffer));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$get_f32(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<float > buffer, ::std::size_t *return$) noexcept {
  ::std::size_t (::NativeIndex::*get_f32$)(::std::uint64_t, ::rust::Slice<float >) const = &::NativeIndex::get_f32;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::size_t((self.*get_f32$)(key, buffer));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$get_f64(::NativeIndex const &self, ::std::uint64_t key, ::rust::Slice<double > buffer, ::std::size_t *return$) noexcept {
  ::std::size_t (::NativeIndex::*get_f64$)(::std::uint64_t, ::rust::Slice<double >) const = &::NativeIndex::get_f64;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::size_t((self.*get_f64$)(key, buffer));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$remove(::NativeIndex const &self, ::std::uint64_t key, ::std::size_t *return$) noexcept {
  ::std::size_t (::NativeIndex::*remove$)(::std::uint64_t) const = &::NativeIndex::remove;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::size_t((self.*remove$)(key));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$rename(::NativeIndex const &self, ::std::uint64_t from, ::std::uint64_t to, ::std::size_t *return$) noexcept {
  ::std::size_t (::NativeIndex::*rename$)(::std::uint64_t, ::std::uint64_t) const = &::NativeIndex::rename;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        new (return$) ::std::size_t((self.*rename$)(from, to));
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

bool cxxbridge1$192$NativeIndex$contains(::NativeIndex const &self, ::std::uint64_t key) noexcept {
  bool (::NativeIndex::*contains$)(::std::uint64_t) const = &::NativeIndex::contains;
  return (self.*contains$)(key);
}

::std::size_t cxxbridge1$192$NativeIndex$count(::NativeIndex const &self, ::std::uint64_t key) noexcept {
  ::std::size_t (::NativeIndex::*count$)(::std::uint64_t) const = &::NativeIndex::count;
  return (self.*count$)(key);
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$save(::NativeIndex const &self, ::rust::Str path) noexcept {
  void (::NativeIndex::*save$)(::rust::Str) const = &::NativeIndex::save;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*save$)(path);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$load(::NativeIndex const &self, ::rust::Str path) noexcept {
  void (::NativeIndex::*load$)(::rust::Str) const = &::NativeIndex::load;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*load$)(path);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$view(::NativeIndex const &self, ::rust::Str path) noexcept {
  void (::NativeIndex::*view$)(::rust::Str) const = &::NativeIndex::view;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*view$)(path);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$reset(::NativeIndex const &self) noexcept {
  void (::NativeIndex::*reset$)() const = &::NativeIndex::reset;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*reset$)();
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::std::size_t cxxbridge1$192$NativeIndex$memory_usage(::NativeIndex const &self) noexcept {
  ::std::size_t (::NativeIndex::*memory_usage$)() const = &::NativeIndex::memory_usage;
  return (self.*memory_usage$)();
}

char const *cxxbridge1$192$NativeIndex$hardware_acceleration(::NativeIndex const &self) noexcept {
  char const *(::NativeIndex::*hardware_acceleration$)() const = &::NativeIndex::hardware_acceleration;
  return (self.*hardware_acceleration$)();
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$save_to_buffer(::NativeIndex const &self, ::rust::Slice<::std::uint8_t > buffer) noexcept {
  void (::NativeIndex::*save_to_buffer$)(::rust::Slice<::std::uint8_t >) const = &::NativeIndex::save_to_buffer;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*save_to_buffer$)(buffer);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$load_from_buffer(::NativeIndex const &self, ::rust::Slice<::std::uint8_t const> buffer) noexcept {
  void (::NativeIndex::*load_from_buffer$)(::rust::Slice<::std::uint8_t const>) const = &::NativeIndex::load_from_buffer;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*load_from_buffer$)(buffer);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}

::rust::repr::PtrLen cxxbridge1$192$NativeIndex$view_from_buffer(::NativeIndex const &self, ::rust::Slice<::std::uint8_t const> buffer) noexcept {
  void (::NativeIndex::*view_from_buffer$)(::rust::Slice<::std::uint8_t const>) const = &::NativeIndex::view_from_buffer;
  ::rust::repr::PtrLen throw$;
  ::rust::behavior::trycatch(
      [&] {
        (self.*view_from_buffer$)(buffer);
        throw$.ptr = nullptr;
      },
      ::rust::detail::Fail(throw$));
  return throw$;
}
} // extern "C"

bool IndexOptions::operator==(IndexOptions const &rhs) const noexcept {
  return cxxbridge1$192$IndexOptions$operator$eq(*this, rhs);
}

bool IndexOptions::operator!=(IndexOptions const &rhs) const noexcept {
  return cxxbridge1$192$IndexOptions$operator$ne(*this, rhs);
}

extern "C" {
static_assert(::rust::detail::is_complete<::std::remove_extent<::NativeIndex>::type>::value, "definition of `::NativeIndex` is required");
static_assert(sizeof(::std::unique_ptr<::NativeIndex>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::NativeIndex>) == alignof(void *), "");
void cxxbridge1$unique_ptr$NativeIndex$null(::std::unique_ptr<::NativeIndex> *ptr) noexcept {
  ::new (ptr) ::std::unique_ptr<::NativeIndex>();
}
void cxxbridge1$unique_ptr$NativeIndex$raw(::std::unique_ptr<::NativeIndex> *ptr, ::std::unique_ptr<::NativeIndex>::pointer raw) noexcept {
  ::new (ptr) ::std::unique_ptr<::NativeIndex>(raw);
}
::std::unique_ptr<::NativeIndex>::element_type const *cxxbridge1$unique_ptr$NativeIndex$get(::std::unique_ptr<::NativeIndex> const &ptr) noexcept {
  return ptr.get();
}
::std::unique_ptr<::NativeIndex>::pointer cxxbridge1$unique_ptr$NativeIndex$release(::std::unique_ptr<::NativeIndex> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge1$unique_ptr$NativeIndex$drop(::std::unique_ptr<::NativeIndex> *ptr) noexcept {
  ::rust::deleter_if<::rust::detail::is_complete<::NativeIndex>::value>{}(ptr);
}
} // extern "C"
