# Agent-related browser extension inventory

**Inspection date:** 2026-09-02  
**Scope:** the standard local user-data locations for the installed Chrome, Edge, Safari, and Firefox applications on this Mac. Profile display names and account identities are intentionally omitted. No browsing history, cookies, credentials, visited URLs, open tabs, page content, or extension-private storage was read.

## Result

One in-scope browser component was observed: Microsoft Edge ships an **Edge Copilot Bridge** component in every inspected Edge profile. It is a browser-bundled component, not a user-installed store extension. No installed OMP Browser Relay, Claude, ChatGPT, general MCP, or coding-agent browser extension was observed in the inspected Chrome, Edge, or Safari profiles. Firefox is not installed and has no standard local profile tree on this machine.

| Browser / anonymized scope | In-scope observation | Version | State supported by metadata | Installation / policy owner | Related host-side component |
| --- | --- | --- | --- | --- | --- |
| Chrome C1 (one profile) | No matching installed record. The official Claude (`fcoeoabgfenejglbffodgkkbkcdhcgfn`) and ChatGPT Search (`ejcfepkfckglbgocfkanmcdngdijcgld`) IDs are absent. Installed payload manifests contain no OMP, Claude, ChatGPT, Copilot, MCP, relay, or agent match. | — | Absent from this profile snapshot, not globally proven absent | No Chrome extension-management policy file was present in the inspected managed-preference locations | The `omp` CLI is installed, but its default relay-extension install directory is absent. A Notion native-host registration exists, but its allowed extension ID has no installed Chrome record; the registration is not counted as an extension. |
| Edge E1, E2 | **Edge Copilot Bridge**, bundled component | `152.0.4191.52` | Enabled in the preference snapshot: `disable_reasons` is empty | Microsoft Edge application bundle (`location: 5`, `from_webstore: false`); not policy-installed | Microsoft Edge and its locally cached **Edge Sidebar** component (`2026.2.24.1`) |
| Edge E3 | **Edge Copilot Bridge**, bundled component | `151.0.4129.100` | Enabled in the preference snapshot: `disable_reasons` is empty | Microsoft Edge application bundle (`location: 5`, `from_webstore: false`); not policy-installed | Microsoft Edge and its locally cached **Edge Sidebar** component (`2026.2.24.1`) |
| Safari (machine discovery; profile identities not read) | No in-scope extension. `pluginkit` reports one Safari web extension, 1Password `8.12.34`, which is unrelated and is mentioned only to establish that discovery was working. | — | Safari profile-specific enablement was not exposed by the safe machine-level listing | No Safari extension-management preference file was present in the inspected managed-preference locations | No in-scope containing app or Safari `.appex` was observed |
| Firefox | Firefox application absent; standard Firefox support/profile directory absent | — | Not applicable | No Firefox extension-policy preference file was present in the inspected managed-preference locations | A system native-host file alone is not a Firefox installation or extension record |

### Edge classification

The Copilot finding is deliberately classified as **bundled**, not **installed from a store**. Each profile preference record points into `Microsoft Edge.app/.../Resources/edge_copilot_bridge_extension`, uses component location `5`, reports `from_webstore: false`, and has no disable reason. Two profiles reference the current Edge application generation; the least recently used profile retains an older application-generation record. The application-support tree also contains an `Edge Sidebar/2026.2.24.1` component cache with Copilot image assets. That cache corroborates that this Edge installation has shipped Copilot sidebar material, but a cache is not an extension installation or proof that the Copilot UI is currently shown.

Microsoft documents Copilot as a built-in Edge sidebar app and explains that sidebar apps can have extension IDs. That is why an internal bridge manifest must not be reported as a user-installed Copilot extension.[^edge-sidebar]

### OMP Browser Relay

OMP documents Browser Relay as an MV3 Chrome extension paired with a loopback relay and installed by `omp browser-relay install`; the local CLI help gives the default extension directory as `~/.omp/browser-relay/extension`.[^omp-browser]

Local evidence shows:

- `/opt/homebrew/bin/omp` exists and exposes the `browser-relay serve|install` commands;
- the default extension directory and its `manifest.json` do not exist;
- no installed Chrome or Edge manifest name matches OMP or Browser Relay;
- no matching record appears in the inspected Chromium profile extension metadata.

Conclusion: the **host CLI is available, but the relay extension is not installed in the inspected browser profiles**. A custom `omp browser-relay install --dir=...` location cannot be excluded in isolation, but it would still need a corresponding loaded extension record; none was observed.

### Claude, ChatGPT, Copilot, and MCP checks

- **Claude:** Anthropic identifies its Chrome extension by ID `fcoeoabgfenejglbffodgkkbkcdhcgfn`; that ID is absent from all inspected Chrome and Edge profiles.[^claude]
- **ChatGPT:** OpenAI identifies its official ChatGPT Search extension by ID `ejcfepkfckglbgocfkanmcdngdijcgld`; that ID is absent from all inspected Chrome and Edge profiles.[^chatgpt]
- **Copilot:** no user/store-installed Copilot extension was observed. Edge's bundled Copilot Bridge is the sole positive finding.
- **MCP:** no installed extension manifest contains an MCP or agent-control identity. MCP server configuration was not treated as browser-extension evidence; MCP's architecture distinguishes hosts, clients, and servers.[^mcp]
- **Desktop applications are not extensions:** locally installed Claude and ChatGPT application bundles were not counted as browser integrations.

## Evidence and method

### Local inspection

1. Confirmed application bundles with macOS metadata: Chrome, Edge, and Safari are installed; Firefox is not.
2. Enumerated only standard profile directories, Chromium `Extensions/<id>/<version>/manifest.json` payloads, and the extension portion of Chromium `Secure Preferences`. Profile names were replaced with C1/E1/E2/E3 in this report.
3. Used manifest name/version, payload path, component location, web-store flag, and `disable_reasons` only. Chromium currently derives enablement from an empty disable-reason set; `running` or service-worker activity was not used as an enablement proxy.[^chromium-prefs][^disable-reasons]
4. Queried the two official Chrome Web Store IDs directly and searched installed manifest names for OMP, relay, Claude, ChatGPT, Copilot, MCP, and agent terms.
5. Inspected managed-preference locations without reading unrelated policy values. Edge has a managed preference file, but its extension force-list/settings/block/allow keys are unset. Chrome, Firefox, and Safari browser extension policy files were not present in the inspected locations.
6. Used `pluginkit` to enumerate Safari web-extension bundles and their containing app metadata. No Safari UI, tabs, pages, website permissions, or private-browsing settings were opened.
7. Inspected native-messaging-host **metadata only**. Chrome has a user-scoped `com.notion.meeting_notes` host owned by `Notion.app`, allowing extension ID `lakhlegpnjnhhgnneabgjkbpbgijibkp`; that ID is absent from C1. A host registration is therefore negative/corollary evidence, not an installed extension.

Chrome's public management API defines the runtime fields `enabled`, `disabledReason`, and `installType` for installed extensions.[^chrome-management] This inspection intentionally did not install a management-capable probe extension. The closed-file evidence is grounded instead in current Chromium preference implementation plus on-disk manifests; it is a snapshot, not a substitute for the browser runtime API.

### State taxonomy

| Evidence | Meaning used here |
| --- | --- |
| Valid profile record plus manifest/payload | Installed or browser-bundled record, depending on location/provenance |
| Component location pointing into the browser app bundle | Bundled browser component; not a user/store installation |
| Nonempty Chromium `disable_reasons` | Disabled for one or more reasons; codes are preserved rather than guessed |
| Empty Chromium `disable_reasons` on a valid installed/component record | Enabled in this preference snapshot |
| Force-install / extension-settings policy entry | Desired management state only; not proof installation completed |
| Sidebar/component cache or native-host registration | Supporting artifact only; not an installed extension |
| No matching record in an inspected profile | Negative evidence for that profile/snapshot only |

Chrome and Edge document force-install and settings policies separately from installed runtime state.[^chrome-force][^chrome-settings][^edge-force][^edge-settings] No matching policy record was observed here.

## Gaps and confidence

- **High confidence** in the positive Edge Copilot Bridge classification: profile records, bundle paths, versions, provenance fields, and the first-party Edge sidebar documentation agree.
- **High confidence** that the official Claude and ChatGPT Search IDs are absent from the inspected Chromium profiles.
- **Moderate-to-high confidence** that OMP Browser Relay is not loaded in the inspected profiles: neither an installed record nor the default payload exists. A manually retained payload in an arbitrary directory is possible, but a payload alone would not be a loaded extension.
- Chromium profiles were inspected as live on-disk snapshots. A browser update left E3's component record on an older generation; this is reported rather than normalized away.
- Safari web-extension bundle discovery is machine-wide, while enablement is profile-specific. Apple states that each non-default Safari profile manages extensions independently and starts with extensions off.[^safari-enable] Because the safe listing did not expose those switches, no Safari enablement claim is made.
- Firefox could exist in a nonstandard path or another macOS account. This report establishes absence from `/Applications` and this user's standard Firefox support location only.
- Declarative device-management state outside the inspected managed-preference files was not exhaustively decoded. Policy absence therefore means “no record in the inspected policy sources,” not “this Mac has no management.”

## First-party sources

[^chrome-management]: Google Chrome, [chrome.management API](https://developer.chrome.com/docs/extensions/reference/api/management).
[^chromium-prefs]: Chromium source, [`extension_prefs.cc`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/extensions/browser/extension_prefs.cc).
[^disable-reasons]: Chromium source, [`disable_reason.h`](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/extensions/browser/disable_reason.h).
[^chrome-force]: Google Chrome Enterprise, [ExtensionInstallForcelist](https://chromeenterprise.google/policies/extension-install-forcelist/).
[^chrome-settings]: Google Chrome Enterprise, [ExtensionSettings](https://chromeenterprise.google/policies/extension-settings/).
[^edge-force]: Microsoft Edge, [ExtensionInstallForcelist](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/extensioninstallforcelist).
[^edge-settings]: Microsoft Edge, [ExtensionSettings](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-policies/extensionsettings).
[^edge-sidebar]: Microsoft, [Manage the sidebar in Microsoft Edge](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-sidebar).
[^safari-enable]: Apple, [Use Safari extensions on your Mac](https://support.apple.com/en-us/102343) and [Safari web extensions](https://developer.apple.com/documentation/safariservices/safari-web-extensions).
[^claude]: Anthropic, [Get started with Claude in Chrome](https://support.claude.com/en/articles/12012173-get-started-with-claude-in-chrome).
[^chatgpt]: OpenAI, [ChatGPT Search](https://help.openai.com/en/articles/9237897-chatgpt-search).
[^omp-browser]: Oh My Pi, [browser tool documentation](https://github.com/can1357/oh-my-pi/blob/main/docs/tools/browser.md).
[^mcp]: Model Context Protocol, [Architecture](https://modelcontextprotocol.io/specification/2026-07-28/architecture).
