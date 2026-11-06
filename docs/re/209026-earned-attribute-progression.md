# Build 209026: explicit earned-attribute progression

The [starter baseline](209026-starter-base-attributes.md) populated the four
primary attributes, but the XP path still changed only earned level. Crossing
2,000 XP therefore sent level 2 while leaving earned attributes at 37. A
waypoint rejoin recreated that mismatch after restoring the earned level.

`--mission-base-attributes default-curve` now explicitly selects the frozen
[default primary-attribute curve](209026-default-primary-attribute-curve.md).
It starts earned/effective selectors 1–4 at 37, with the separate signed
column zero, just as the earlier fixed37 profile did. On earned-level changes
it sends four server187 updates after server188: level 2 receives 44, level 3
receives 51, and level 80 receives 1000. Level skips choose the reached row.
Rejoin restores the earned values after restoring XP and earned level.

This option is an unmodified profile model: no gear, account flag or other
modifier is inferred. The verified table contains all 101 source rows; the
player lookup accepts only levels 1–80. Existing numeric settings retain
their fixed values and do not acquire an inferred progression curve. Omitting
the setting preserves the previous population and XP output.

## Effective state remains a separate requirement

Server187 updates the earned column. The earned-level update does not establish
the policy selecting the second level or effective attributes, so this change
does not send189 on level-up or alter health. In this explicit profile the
second level and effective baseline remain at their initial level-one values.
That remaining limitation is not claimed to reproduce retail scaling. Skill
execution, localization keys and authored Great Hunt encounters also remain
incomplete. XP and heart state still reset when the server restarts.

## Configuration and checks

The existing supervisor forwards `GW2_BASE_ATTRIBUTES=default-curve` to the
world role and its preflight. It changes no saved/prepared roster selection.
Candidate profile:
`~/re/gw2/notes/mission-profile-attribute-curve-20261106.conf`, SHA256
**d7e25b1da72a9354f7cf84c7d46ad893158fac3c24a1cda13a68d73a6a9ee3ef**.

Compared with the pinned cdn900 profile, this adds only that setting and removes
the retired `GW2_AUTOATTACK_SKILL=10170`, preserving all other values. The
baseline and earlier fixed37 candidate remain untouched. Exact paths and pins
are in `~/re/gw2/notes/attribute-curve-profile-20261106.json`.

- Two curve tests pass, and REA verified all 101 module constants against the
  pinned decoded source table. No new archive extraction was needed.
- Four focused base-attribute tests pass: explicit parsing, exact startup
  bytes, saved GUID selection, threshold crossing, saturation and fixed/omitted
  compatibility.
- All 39 mission tests pass. The isolated waypoint lifecycle checks ordinary
  kill XP, heart XP and restored earned values, with no effective-column
  updates and no repeated heart reward.
- All seven saved-population tests and the no-listener saved configuration
  test pass. The latter accepts fixed0/37/65535 and the explicit curve.
- Three focused supervisor tests pass for default behavior and prepared/saved
  forwarding. These use fake children and do not start game listeners.

These are offline server-output checks. No new shipped-client observation or
online deployment is claimed. The sole publisher's native owner question,
run145 hold and resource requirements remain unchanged.
