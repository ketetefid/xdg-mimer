Name:           xdg-mimer
Version:        @VERSION@
Release:        1%{?dist}
Summary:        A simple GUI tool for MIME associations
License:        GPLv3
URL:            https://github.com/ketetefid/xdg-mimer
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust
Requires:       glibc

%description
xdg-mimer is a GUI tool to view or change MIME associations using XDG standards.

%prep
%autosetup

%build
cargo build --release

%install
install -D target/release/xdg-mimer %{buildroot}/usr/bin/xdg-mimer

%files
/usr/bin/xdg-mimer

%changelog
* Fri Jul 05 2025 Kete Tefid <ketetefid@gmail.com> - @VERSION@-1
- Initial RPM release
