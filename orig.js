/**
 * Mod file parser
 * @class
 */
class modFileChecker {
	#maxFilesType = { grle : 10, pdf : 1, png : 128, txt : 2 }
	#fileSizeMap  = { cache : 10485760, dds : 12582912, gdm : 18874368, shapes : 268435456, xml : 262144 }

	#flag_broken = {
		FILE_ERROR_GARBAGE_FILE                : false,
		FILE_ERROR_LIKELY_COPY                 : false,
		FILE_ERROR_LIKELY_ZIP_PACK             : false,
		FILE_ERROR_NAME_INVALID                : false,
		FILE_ERROR_NAME_STARTS_DIGIT           : false,
		FILE_ERROR_UNREADABLE_ZIP              : false,
		FILE_ERROR_UNSUPPORTED_ARCHIVE         : false,
		MOD_ERROR_NO_MOD_VERSION               : false,
		NOT_MOD_MODDESC_MISSING                : false,
		NOT_MOD_MODDESC_PARSE_ERROR            : false,
		NOT_MOD_MODDESC_VERSION_OLD_OR_MISSING : false,
	}

	#flag_info = {
		INFO_NO_MULTIPLAYER_UNZIPPED           : false,
		FILE_IS_A_SAVEGAME                     : false,
		MALICIOUS_CODE                         : false,
	}

	#flag_problem = {
		INFO_MIGHT_BE_PIRACY                   : false,
		MOD_ERROR_MODDESC_DAMAGED_RECOVERABLE  : false,
		MOD_ERROR_NO_MOD_ICON                  : false,
		PERF_DDS_TOO_BIG                       : false,
		PERF_GDM_TOO_BIG                       : false,
		PERF_GRLE_TOO_MANY                     : false,
		PERF_HAS_EXTRA                         : false,
		PERF_I3D_TOO_BIG                       : false,
		PERF_L10N_NOT_SET                      : false,
		PERF_PDF_TOO_MANY                      : false,
		PERF_PNG_TOO_MANY                      : false,
		PERF_SHAPES_TOO_BIG                    : false,
		PERF_SPACE_IN_FILE                     : false,
		PERF_TXT_TOO_MANY                      : false,
		PERF_XML_TOO_BIG                       : false,
		PREF_PNG_TEXTURE                       : false,
	}

	modDesc = {
		actions        : {},
		author         : '--',
		binds          : {},
		cropInfo       : false,
		cropWeather    : null,
		depend         : [],
		descVersion    : 0,
		iconFileName   : false,
		iconImageCache : null,
		mapConfigFile  : null,
		mapIsSouth     : false,
		multiPlayer    : false,
		scriptFiles    : 0,
		storeItems     : 0,
		version        : '--',
	}

	modDescRAW    = null
	modDescParsed = false

	md5Sum            = null
	uuid              = null
	currentCollection = null

	fileDetail = {
		copyName    : false,
		extraFiles  : [],
		fileDate    : null,
		fileSize    : 0,
		fullPath    : false,
		i3dFiles    : [],
		imageDDS    : [],
		imageNonDDS : [],
		isFolder    : false,
		isSaveGame  : false,
		pngTexture  : [],
		shortName   : false,
		spaceFiles  : [],
		tooBigFiles : [],
	}

	canNotUse     = true
	badges        = ''
	currentLocale = null

	#l10n           = null
	#locale         = null
	#log            = null
	#iconParser     = requiredItems.iconDecoder

	#modHandle = null
	
	/**
	 * Create new mod parsing instance
	 * @param {string} filePath Path to mod file/folder
	 * @param {boolean} isFolder Mod is a folder
	 * @param {number} size Size of mod
	 * @param {Date} date Date of mod
	 * @param {modRecord_md5Sum} md5Pre MD5 hash from location and date
	 */
	constructor(filePath, isFolder, size, date, md5Pre = null) {
		this.fileDetail.fullPath = filePath
		this.fileDetail.isFolder = isFolder
		this.fileDetail.fileSize = size
		this.fileDetail.fileDate = date
		
		this.md5Sum    = md5Pre ?? null
		this.#locale   = requiredItems.currentLocale

		this.fileDetail.shortName = path.parse(this.fileDetail.fullPath).name

		this.#flag_info.INFO_NO_MULTIPLAYER_UNZIPPED = this.fileDetail.isFolder
	}

	/**
	 * Actually parse the mod
	 * @returns {modRecord_storable}
	 */
	async getInfo() {
		this.uuid      = crypto.createHash('md5').update(this.fileDetail.fullPath).digest('hex')
		this.#log      = new logCollector(`mod-${this.uuid}`)
		this.#log.info(`Adding Mod File: ${this.fileDetail.shortName}`)
			
		const isValidMod = this.#doStep_validFileName

		try {
			if ( !isValidMod ) {
				this.#util_raiseFlag_broken('FILE_ERROR_NAME_INVALID')
			}

			this.#modHandle = new fileHandlerAsync(this.fileDetail.fullPath, this.fileDetail.isFolder, this.#log)

			const couldOpen = await this.#modHandle.open()

			if ( !couldOpen ) {
				if ( !isValidMod ) { throw new Error('Invalid Mod') }
				this.#util_raiseFlag_broken('FILE_ERROR_UNREADABLE_ZIP')
				throw new Error('Unreadable ZIP File')
			}
				
			if ( this.#modHandle.exists('careerSavegame.xml')) {
				this.fileDetail.isSaveGame = true
				this.modDesc.version       = '--'
				this.#util_raiseFlag_info('FILE_IS_A_SAVEGAME')
				throw new Error('Savegame Detected')
			}

			if ( !isValidMod ) {
				throw new Error('Invalid Mod')
			}

			if ( ! this.#modHandle.exists('modDesc.xml') ) {
				this.#util_raiseFlag_broken('NOT_MOD_MODDESC_MISSING')
				this.md5Sum                 = null
				throw new Error('ModDesc Missing, Invalid, or Un-Readable')
			}

			await this.#doStep_fileCounts()
			await this.#doStep_parseModDesc()
				
			if ( this.modDesc.mapConfigFile !== null ) {
				try {
					if (! this.#modHandle.exists(this.modDesc.mapConfigFile) ) { throw new Error('Config file does not Exist')}

					const cropConfigFiles = await this.#doStep_parseMapXML(this.modDesc.mapConfigFile)

					const cropInfo = new cropDataReader(
						await this.#modHandle.readText(cropConfigFiles[0]),
						await this.#modHandle.readText(cropConfigFiles[1]),
						await this.#modHandle.readText(cropConfigFiles[2]),
						cropConfigFiles[3]
					)

					this.modDesc.cropWeather = cropInfo.weather
					this.modDesc.cropInfo    = cropInfo.crops
					this.modDesc.mapIsSouth  = cropInfo.isSouth
				} catch (err) {
					this.#log.notice(`Caught map fail: ${err.message}`)
				}
			}

			try {
				if ( this.#flag_problem.MOD_ERROR_NO_MOD_ICON || typeof this.modDesc.iconFileName !== 'string' || ! this.#modHandle.exists(this.modDesc.iconFileName) ) {
					throw new Error('File does not Exist')
				}

				this.modDesc.iconImageCache = await this.#iconParser.parseDDS(
					await this.#modHandle.readBin(this.modDesc.iconFileName),
					false
				)
			} catch (err) {
				this.#util_raiseFlag_problem('MOD_ERROR_NO_MOD_ICON')
				this.#log.notice(`Caught icon fail: ${err.message}`)
			}
		} catch (err) {
			this.#log.notice(`Stopping Mod Parse : ${err.message}`)
		} finally {
			this.#doStep_l10n()
			this.#modHandle?.close?.()
		}
		this.#modHandle    = null
		this.modDescParsed = null
		this.modDescRAW    = null

		return this.storable
	}

	/**
	 * @type {modRecord_storable}
	 */
	get storable() {
		return {
			log    : this.#log.lines,
			record : {
				badgeArray        : this.#doStep_badges,
				canNotUse         : this.#doStep_canUse,
				currentCollection : this.currentCollection,
				fileDetail        : this.fileDetail,
				issues            : Object.entries({...this.#flag_broken, ...this.#flag_problem, ...this.#flag_info}).filter((x) => x[1] === true).map((x) => x[0]),
				l10n              : this.#l10n,
				md5Sum            : this.md5Sum,
				modDesc           : this.modDesc,
				uuid              : this.uuid,
			},
		}
	}

	async #doStep_fileCounts() {
		for ( const checkFile of this.#modHandle.list ) {
			
			const fileInfo = this.#modHandle.fileInfo(checkFile)
			const fileName = checkFile
			const fullName = this.fileDetail.isFolder ? this.#modHandle.relativeFolder(checkFile) : checkFile

			if ( fileName.includes(' ') ) {
				this.fileDetail.spaceFiles.push(fileName)
				this.#util_raiseFlag_problem('PERF_SPACE_IN_FILE')
			}

			if ( fileInfo.isFolder ) { continue }

			/* eslint-disable no-await-in-loop */
			await this.#util_countFile(
				path.extname(fileName),
				fileName,
				fileInfo.size,
				fullName
			)
			/* eslint-enable no-await-in-loop */
		}

		this.#flag_problem.PERF_GRLE_TOO_MANY = ( this.#maxFilesType.grle < 1 )
		this.#flag_problem.PERF_PNG_TOO_MANY  = ( this.#maxFilesType.png < 1 )
		this.#flag_problem.PERF_PDF_TOO_MANY  = ( this.#maxFilesType.pdf < 1 )
		this.#flag_problem.PERF_TXT_TOO_MANY  = ( this.#maxFilesType.txt < 1 )
	}

	get #doStep_canUse() {
		return this.fileDetail.isSaveGame ? true : Object.entries(this.#flag_broken).some((x) => x[1] === true)
	}

	#doStep_l10n() {
		const title = this.#modDesc_localString('title', '--')
		const desc  = this.#modDesc_localString('description')

		this.#flag_info.PERF_L10N_NOT_SET  = ( title === '--' || desc === '' )

		this.#l10n = {
			title       : title,
			description : desc,
		}
	}

	get #doStep_badges() {
		const badges = {
			broken  : this.fileDetail.isSaveGame ? false : Object.entries(this.#flag_broken).some((x) => x[1] === true),
			folder  : this.fileDetail.isFolder,
			malware : this.#flag_info.MALICIOUS_CODE,
			noMP    : ! this.modDesc.multiPlayer && this.fileDetail.isFolder,
			notmod  : this.#flag_broken.NOT_MOD_MODDESC_MISSING,
			pconly  : (this.modDesc.scriptFiles > 0),
			problem : this.fileDetail.isSaveGame ? false : Object.entries(this.#flag_problem).some((x) => x[1] === true),
			savegame : this.fileDetail.isSaveGame,
		}

		return Object.entries(badges).filter((x) => x[1] === true).map((x) => x[0])
	}

	get #doStep_validFileName() {
		const fullModPath = this.fileDetail.fullPath
		const shortName   = this.fileDetail.shortName

		if ( ! this.fileDetail.isFolder && ! fullModPath.endsWith('.zip') ) {
			if ( fullModPath.endsWith('.rar') || fullModPath.endsWith('.7z') ) {
				this.#util_raiseFlag_broken('FILE_ERROR_UNSUPPORTED_ARCHIVE')
			} else {
				this.#util_raiseFlag_broken('FILE_ERROR_GARBAGE_FILE')
			}
			return false
		}

		if ( shortName.match(/unzip/i) ) {
			this.#util_raiseFlag_broken('FILE_ERROR_LIKELY_ZIP_PACK')
		}

		if ( shortName.match(/^\d/) ) {
			this.#util_raiseFlag_broken('FILE_ERROR_NAME_STARTS_DIGIT')
			return false
		}

		if ( ! shortName.match(/^[A-Z_a-z]\w+$/) ) {
			const copyName = shortName.match(/^([A-Za-z]\w+)(?: - .+$| \(.+$)/)

			if ( copyName !== null ) {
				this.#util_raiseFlag_broken('FILE_ERROR_LIKELY_COPY')
				this.fileDetail.copyName    = copyName[1]
			}
			return false
		}
		return true
	}

	#modDesc_localString(key, fallback = '') {
		const searchTree = this.modDescParsed?.[key.toLowerCase()] ?? fallback

		if ( searchTree === null ) { return fallback }

		try {
			return searchTree?.[this.#locale] ?? searchTree?.en ?? searchTree?.de ?? fallback
		} catch (err) {
			this.#log.warning(`Caught odd entry: ${key} :: ${err}`)
			return fallback
		}
	}

	#modDesc_default(key, fallback = null) { return key ?? fallback }

	/* eslint-disable-next-line complexity */
	async #doStep_parseModDesc() {
		this.modDescParsed = await this.#modHandle.readXML('modDesc.xml', 'moddesc', 'moddesc')
		
		if ( this.modDescParsed === false ) {
			this.#util_raiseFlag_broken('NOT_MOD_MODDESC_PARSE_ERROR')
			return
		}

		if ( this.modDescParsed === null ) {
			this.#util_raiseFlag_broken('NOT_MOD_MODDESC_MISSING')
			return
		}

		/* Get modDesc.xml version */
		this.modDesc.descVersion   = this.#modDesc_default(this.modDescParsed?.$?.DESCVERSION, 0)
		this.#flag_broken.NOT_MOD_MODDESC_VERSION_OLD_OR_MISSING = ( this.modDesc.descVersion === 0 )
	
		/* Get MOD Version */
		this.modDesc.version       = this.#modDesc_default(this.modDescParsed?.version?.toString?.(), '0.0.0.0')
		this.#flag_broken.MOD_ERROR_NO_MOD_VERSION = ( this.modDesc.version === '0.0.0.0' )

		this.modDesc.author        = this.#modDesc_default(this.modDescParsed?.author, '--')
		this.modDesc.multiPlayer   = this.#modDesc_default(this.modDescParsed?.multiplayer?.$?.SUPPORTED, false)
		this.modDesc.storeItems    = this.#modDesc_default(this.modDescParsed?.storeitems?.storeitem?.length, 0)
		this.modDesc.mapConfigFile = this.#modDesc_default(this.modDescParsed?.maps?.map?.[0]?.$?.CONFIGFILENAME)
		this.modDesc.depend        = this.#modDesc_default(this.modDescParsed?.dependencies?.dependency, [])
		
		this.#flag_problem.INFO_MIGHT_BE_PIRACY = Object.hasOwn(this.modDescParsed, 'productid')

		/* Get icon filename */
		let iconFileName = ''
		
		if ( typeof this.modDescParsed?.iconfilename ==='string' ) {
			iconFileName = this.modDescParsed.iconfilename
		} else if ( typeof this.modDescParsed?.iconfilename?.[0] ==='string' ) {
			iconFileName = this.modDescParsed.iconfilename[0]
		}
		
		if ( typeof iconFileName?.endsWith === 'function' && ! iconFileName.endsWith('.dds') ) {
			iconFileName = `${iconFileName.slice(0, -4)}.dds`
		}

		if ( this.fileDetail.imageDDS.includes(iconFileName) ) {
			this.modDesc.iconFileName = iconFileName
		} else {
			
			this.#util_raiseFlag_problem('MOD_ERROR_NO_MOD_ICON')
		}
		
		this.#doStep_parseActions()
	}

	#doStep_parseActions() {
		try {
			if ( Array.isArray(this.modDescParsed?.actions?.action) ) {
				for ( const action of this.modDescParsed.actions.action ) {
					this.modDesc.actions[action.$.NAME] = action.$.CATEGORY || 'ALL'
				}
			}
			if ( Array.isArray(this.modDescParsed?.inputbinding?.actionbinding) ) {
				for ( const action of this.modDescParsed.inputbinding.actionbinding ) {
					const thisActionName = action.$.ACTION

					if ( Array.isArray(action.binding) ) {
						for ( const binding of action.binding ) {
							if ( binding.$.DEVICE === 'KB_MOUSE_DEFAULT' ) {
								this.modDesc.binds[thisActionName] ??= []
								this.modDesc.binds[thisActionName].push(binding.$.INPUT)
							}
						}
					}
				}
			}
		} catch (err) {
			this.#log.warning(`Key binding read failed : ${err}`)
		}
	}

	async #doStep_parseMapXML(fileName) {
		const mapConfigParsed = await this.#modHandle.readXML(fileName, 'moddesc', 'map')
		
		if ( mapConfigParsed === null ) {
			this.#log.warning('Map XML Files not found')
		}

		return [
			this.#util_nullBaseGameFile(mapConfigParsed?.fruittypes?.$?.FILENAME),
			this.#util_nullBaseGameFile(mapConfigParsed?.growth?.$?.FILENAME),
			this.#util_nullBaseGameFile(mapConfigParsed?.environment?.$?.FILENAME),
			this.#util_getBaseGameFile(mapConfigParsed?.environment?.$?.FILENAME),
		]
	}

	#util_raiseFlag_problem(flag) { this.#flag_problem[flag] = true }
	#util_raiseFlag_broken(flag)  { this.#flag_broken[flag] = true }
	#util_raiseFlag_info(flag)    { this.#flag_info[flag] = true }

	#util_size_check(size, type, fileName, flagPart) {
		if ( size > this.#fileSizeMap[type] ) {
			this.fileDetail.tooBigFiles.push(fileName)
			this.#util_raiseFlag_problem(`PERF_${flagPart}_TOO_BIG`)
		}
	}
	#util_tick_count(type) { this.#maxFilesType[type]-- }

	#util_countUnknown(shortSuffix, fileName) {
		const knownGood   = new Set(['', 'png', 'dds', 'i3d', 'shapes', 'lua', 'gdm', 'cache', 'xml', 'grle', 'pdf', 'txt', 'gls', 'anim', 'ogg'])

		if ( !knownGood.has(shortSuffix) ) {
			if ( shortSuffix === 'l64' || shortSuffix === 'dat' ) {
				this.#util_raiseFlag_problem('INFO_MIGHT_BE_PIRACY')
			}
			this.#util_raiseFlag_problem('PERF_HAS_EXTRA')
			this.fileDetail.extraFiles.push(fileName)
			return true
		}
		return false
	}

	async #util_checkLUA(fileName) {
		if ( malwareFalse.has(this.fileDetail.shortName)) { return }

		const luaContents = await this.#modHandle.readText(fileName)

		if ( typeof luaContents !== 'string' ) { return }

		if ( luaContents.match(/\.deleteFolder/) ) {
			this.#util_raiseFlag_info('MALICIOUS_CODE')
		} else if ( luaContents.match(/\.deleteFile/) ) {
			this.#util_raiseFlag_info('MALICIOUS_CODE')
		}
		// console.log(luaContents)
	}

	async #util_countFile(suffix, fileName, size, fullFileName) {
		const shortSuffix = suffix.substring(1)

		if ( this.#util_countUnknown(shortSuffix, fileName) ) { return }

		switch (shortSuffix) {
			case 'png' : {
				this.#util_tick_count('png')
				if ( ! fileName.endsWith('_weight.png') ) {
					this.fileDetail.imageNonDDS.push(fileName)
					this.fileDetail.pngTexture.push(fileName)
				}
				break
			}
			case 'dds' :
				this.fileDetail.imageDDS.push(fileName)
				this.#util_size_check(size, 'dds', fileName, 'DDS')
				break
			case 'i3d' :
				this.fileDetail.i3dFiles.push(fileName)
				break
			case 'lua' :
				this.modDesc.scriptFiles++
				await this.#util_checkLUA(fullFileName)
				break
			case 'cache' :
				this.#util_size_check(size, shortSuffix, fileName, 'I3D')
				break
			case 'gdm' :
			case 'xml' :
			case 'shapes' :
				this.#util_size_check(size, shortSuffix, fileName, shortSuffix.toUpperCase())
				break
			case '.grle' :
			case '.pdf' :
			case '.txt' :
				this.#util_tick_count(shortSuffix)
				break
			default :
				break
		}
		
	}

	#util_nullBaseGameFile(fileName) {
		return ( typeof fileName === 'string' && !fileName.startsWith('$') ) ? fileName : null
	}

	#util_getBaseGameFile(fileName) {
		try {
			return ( typeof fileName === 'string' && !fileName.startsWith('$') ) ? null : path.normalize(path.dirname(fileName.replace('$data', ''))).split(path.sep).slice(-1)
		} catch { return null }
	}
}