require("git"):setup()
require("smart-enter"):setup({
	open_multi = true,
})

require("quicklook"):setup({
	wsl_distro = "Arch",
	quicklook_path = "/mnt/c/Users/zion/AppData/Local/Programs/QuickLook/QuickLook.exe",
	debug = true,
})

require("starship"):setup()

require("zoxide"):setup({
	update_db = true,
})
