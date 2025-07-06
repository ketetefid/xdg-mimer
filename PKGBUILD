pkgname=xdg-mimer
pkgver=@VERSION@
pkgrel=1
pkgdesc="A GUI tool for managing MIME associations using XDG standards"
arch=('x86_64')
url="https://github.com/ketetefid/xdg-mimer"
license=('GPLv3')
depends=()
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release
}

package() {
  install -Dm755 "$srcdir/$pkgname-$pkgver/target/release/xdg-mimer" "$pkgdir/usr/bin/xdg-mimer"
}
