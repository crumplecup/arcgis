# ESRI Permission System Research

## Summary

Researched ESRI's permission model and discovered **79 unique privileges** from configured API keys.

## Sources

### Official Documentation
- [ArcGIS REST API Privileges Reference](https://developers.arcgis.com/rest/users-groups-and-items/privileges/)
- [ArcGIS Enterprise Privileges](https://developers.arcgis.com/documentation/security-and-authentication/reference/privileges/enterprise/)
- [Portal Privileges for Roles](https://enterprise.arcgis.com/en/portal/latest/administer/windows/privileges-for-roles-orgs.htm)

### Discovery Tool
- Created `examples/discover_permissions.rs` - queries `/community/self` to enumerate privileges
- Results saved to `discovered_permissions.txt`

## Discovered Privileges (79 total)

### Features (2)
- `features:user:edit` - Edit features (respects layer permissions)
- `features:user:fullEdit` - Full edit capability

### Portal - Admin (30)
- `portal:admin:assignToGroups`
- `portal:admin:changeUserRoles`
- `portal:admin:createLeavingDisallowedGroup`
- `portal:admin:createReports`
- `portal:admin:createUpdateCapableGroup`
- `portal:admin:deleteGroups`
- `portal:admin:deleteItems`
- `portal:admin:deleteUsers`
- `portal:admin:disableUsers`
- `portal:admin:inviteUsers`
- `portal:admin:manageCollaborations`
- `portal:admin:manageCredits`
- `portal:admin:manageEnterpriseGroups`
- `portal:admin:manageLicenses`
- `portal:admin:manageRoles`
- `portal:admin:manageSecurity`
- `portal:admin:manageUtilityServices`
- `portal:admin:manageWebsite`
- `portal:admin:reassignGroups`
- `portal:admin:reassignItems`
- `portal:admin:shareToOrg` - Share OTHER users' content
- `portal:admin:shareToPublic`
- `portal:admin:updateGroups`
- `portal:admin:updateItemCategorySchema`
- `portal:admin:updateItems`
- `portal:admin:updateMemberCategorySchema`
- `portal:admin:updateUsers`
- `portal:admin:viewGroups`
- `portal:admin:viewItems`
- `portal:admin:viewUsers`

### Portal - Publisher (7)
- `portal:publisher:createDataPipelines`
- `portal:publisher:publishDynamicImagery`
- `portal:publisher:publishFeatures`
- `portal:publisher:publishScenes`
- `portal:publisher:publishServerGPServices`
- `portal:publisher:publishTiledImagery`
- `portal:publisher:publishTiles`

### Portal - User (17)
- `portal:user:addExternalMembersToGroup`
- `portal:user:createGroup`
- `portal:user:createItem`
- `portal:user:createWorkflow`
- `portal:user:invitePartneredCollaborationMembers`
- `portal:user:joinGroup`
- `portal:user:joinNonOrgGroup`
- `portal:user:reassignItems`
- `portal:user:shareGroupToOrg`
- `portal:user:shareGroupToPublic`
- `portal:user:shareToGroup`
- `portal:user:shareToOrg` - Share OWN content
- `portal:user:shareToPublic`
- `portal:user:viewOrgGroups`
- `portal:user:viewOrgItems`
- `portal:user:viewOrgUsers`
- `portal:user:viewTracks`

### Premium - Publisher (4)
- `premium:publisher:createAdvancedNotebooks`
- `premium:publisher:createNotebooks`
- `premium:publisher:rasteranalysis`
- `premium:publisher:scheduleNotebooks`

### Premium - User (18)
- `premium:user:basemaps`
- `premium:user:demographics`
- `premium:user:featurereport`
- `premium:user:geocode`
- `premium:user:geocode:stored`
- `premium:user:geocode:temporary`
- `premium:user:geoenrichment`
- `premium:user:networkanalysis`
- `premium:user:networkanalysis:closestfacility`
- `premium:user:networkanalysis:lastmiledelivery`
- `premium:user:networkanalysis:locationallocation`
- `premium:user:networkanalysis:optimizedrouting`
- `premium:user:networkanalysis:origindestinationcostmatrix`
- `premium:user:networkanalysis:routing`
- `premium:user:networkanalysis:servicearea`
- `premium:user:networkanalysis:snaptoroads`
- `premium:user:networkanalysis:vehiclerouting`
- `premium:user:spatialanalysis`

### OpenData (1)
- `opendata:user:designateGroup`

## Key Distribution by Tier

### ARCGIS_PUBLIC_KEY (1 privilege)
- premium:user:basemaps

### ARCGIS_LOCATION_KEY (15 privileges)
All geocoding, routing, and network analysis permissions

### ARCGIS_SPATIAL_KEY (3 privileges)
- premium:publisher:rasteranalysis
- premium:user:basemaps
- premium:user:spatialanalysis

### ARCGIS_GENERAL_KEY (32 privileges)
All portal user + publisher permissions, features editing

### ARCGIS_ADMIN_KEY (63 privileges)
All portal admin + user + publisher permissions

## Permission Format

```
category:scope:action[:subaction]
```

Examples:
- `portal:user:createItem` - Portal / User-level / Create item action
- `premium:user:networkanalysis:routing` - Premium / User-level / Network analysis / Routing sub-action
- `portal:admin:deleteItems` - Portal / Admin-level / Delete items action

## Additional Privileges from Documentation (Not in Our Keys)

### AI & Assistants
- `portal:user:useAIAssistants`

### Webhooks
- `portal:user:manageWebhooks`
- `portal:admin:manageWebhooks`

### Data Management
- `portal:user:manageDatabases`
- `portal:user:manageRealTimeAnalytics`

### Versioning
- `features:user:manageVersions`

### Content Transfer
- `portal:user:reassignItemsOnTransfer`
- `portal:user:receiveItemsOnTransfer`
- `portal:user:receiveItems`

### Publishing (Extended)
- `portal:publisher:publishVideo`
- `portal:publisher:bulkPublishExcel`
- `premium:publisher:geoEnrichment`

### Admin (Extended)
- `portal:admin:viewLogs`
- `portal:admin:viewAnalytics`
- `portal:admin:manageOpenData`
- `portal:admin:manageServers`
- `portal:admin:linkEnterpriseGroups`
- `portal:admin:manageBulkPublishing`

### Elevation
- `premium:user:elevation`

## Implementation Notes

1. **Permission Enum**: Create enum with ~100 variants covering all known permissions
2. **String Conversion**: Map ESRI string format ↔ enum variants
3. **Default Tiers**: Each permission knows which tier it typically belongs to (for fallback routing)
4. **Method Registry**: Use `inventory` crate to register method → permission mappings at compile time
5. **Routing Logic**: Check specific → group → skeleton key in .env

## Next Steps

1. ✅ Research ESRI documentation
2. ✅ Discover privileges from configured keys
3. Create Permission enum with all discovered + documented privileges
4. Create method registry infrastructure
5. Register Portal methods with required permissions
6. Implement key routing logic in EnvConfig
7. Update ArcGISClient for transparent routing
