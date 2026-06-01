BIN       := target/release/poke
APP       := Poke.app
APP_NAME  := Poke
BUNDLE_ID := com.lucaguidi.poke
ICON_SRC  := icon.png
ICON      := AppIcon.icns

.PHONY: all build app icon clean

all: app

build:
	cargo build --release

$(ICON): $(ICON_SRC)
	rm -rf AppIcon.iconset
	mkdir -p AppIcon.iconset
	sips -z 16   16   $(ICON_SRC) --out AppIcon.iconset/icon_16x16.png        >/dev/null
	sips -z 32   32   $(ICON_SRC) --out AppIcon.iconset/icon_16x16@2x.png     >/dev/null
	sips -z 32   32   $(ICON_SRC) --out AppIcon.iconset/icon_32x32.png        >/dev/null
	sips -z 64   64   $(ICON_SRC) --out AppIcon.iconset/icon_32x32@2x.png     >/dev/null
	sips -z 128  128  $(ICON_SRC) --out AppIcon.iconset/icon_128x128.png      >/dev/null
	sips -z 256  256  $(ICON_SRC) --out AppIcon.iconset/icon_128x128@2x.png   >/dev/null
	sips -z 256  256  $(ICON_SRC) --out AppIcon.iconset/icon_256x256.png      >/dev/null
	sips -z 512  512  $(ICON_SRC) --out AppIcon.iconset/icon_256x256@2x.png   >/dev/null
	sips -z 512  512  $(ICON_SRC) --out AppIcon.iconset/icon_512x512.png      >/dev/null
	sips -z 1024 1024 $(ICON_SRC) --out AppIcon.iconset/icon_512x512@2x.png   >/dev/null
	iconutil -c icns AppIcon.iconset -o $(ICON)
	rm -rf AppIcon.iconset

icon: $(ICON)

app: build $(ICON)
	rm -rf $(APP)
	mkdir -p $(APP)/Contents/MacOS $(APP)/Contents/Resources
	cp $(BIN) $(APP)/Contents/MacOS/$(APP_NAME)
	cp Info.plist $(APP)/Contents/Info.plist
	cp $(ICON) $(APP)/Contents/Resources/$(ICON)
	codesign --force --sign - --identifier $(BUNDLE_ID) $(APP)
	@echo
	@echo "Built $(APP). Run it once via:"
	@echo "    open $(APP) --args --title Hi --message there"
	@echo "then approve the notification prompt. After that, consent persists."
	@echo "For ongoing CLI use:"
	@echo "    ./poke --title Hi --message there"

clean:
	cargo clean
	rm -rf $(APP) $(ICON) AppIcon.iconset
