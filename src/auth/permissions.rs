//! ESRI permission system for ArcGIS Online and Enterprise.
//!
//! This module defines all ESRI privileges as a strongly-typed enum, enabling
//! type-safe permission checking and automatic error enhancement.

use crate::auth::ApiKeyTier;

/// ESRI permission/privilege for ArcGIS operations.
///
/// Each variant corresponds to an ESRI privilege string (e.g., `"portal:user:createItem"`).
/// Permissions are discovered from API keys via `/community/self` endpoint.
///
/// See [`PERMISSIONS_RESEARCH.md`](../../PERMISSIONS_RESEARCH.md) for complete documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Permission {
    // ============================================================================
    // Features - Service Data Editing (2 privileges)
    // ============================================================================
    /// Edit features (respects layer-level permissions)
    FeaturesUserEdit,

    /// Full edit capability (bypasses layer-level restrictions)
    FeaturesUserFullEdit,

    // ============================================================================
    // Portal - Admin Privileges (30 privileges)
    // ============================================================================
    /// Assign members to groups
    PortalAdminAssignToGroups,

    /// Change user roles
    PortalAdminChangeUserRoles,

    /// Create groups that members cannot leave
    PortalAdminCreateLeavingDisallowedGroup,

    /// Create organization reports
    PortalAdminCreateReports,

    /// Create update-capable groups
    PortalAdminCreateUpdateCapableGroup,

    /// Delete groups
    PortalAdminDeleteGroups,

    /// Delete items
    PortalAdminDeleteItems,

    /// Delete users
    PortalAdminDeleteUsers,

    /// Disable user accounts
    PortalAdminDisableUsers,

    /// Invite users to organization
    PortalAdminInviteUsers,

    /// Manage collaboration agreements
    PortalAdminManageCollaborations,

    /// Manage organization credits
    PortalAdminManageCredits,

    /// Manage enterprise groups
    PortalAdminManageEnterpriseGroups,

    /// Manage user licenses
    PortalAdminManageLicenses,

    /// Create and modify custom roles
    PortalAdminManageRoles,

    /// Manage organization security settings
    PortalAdminManageSecurity,

    /// Manage utility services
    PortalAdminManageUtilityServices,

    /// Manage organization website
    PortalAdminManageWebsite,

    /// Reassign group ownership
    PortalAdminReassignGroups,

    /// Reassign item ownership
    PortalAdminReassignItems,

    /// Share OTHER users' content with organization
    PortalAdminShareToOrg,

    /// Share OTHER users' content publicly
    PortalAdminShareToPublic,

    /// Update group information
    PortalAdminUpdateGroups,

    /// Update item category schema
    PortalAdminUpdateItemCategorySchema,

    /// Update items
    PortalAdminUpdateItems,

    /// Update member category schema
    PortalAdminUpdateMemberCategorySchema,

    /// Update user accounts
    PortalAdminUpdateUsers,

    /// View all groups (including private)
    PortalAdminViewGroups,

    /// View all items
    PortalAdminViewItems,

    /// View all users
    PortalAdminViewUsers,

    // ============================================================================
    // Portal - Publisher Privileges (7 privileges)
    // ============================================================================
    /// Create data pipelines
    PortalPublisherCreateDataPipelines,

    /// Publish dynamic imagery layers
    PortalPublisherPublishDynamicImagery,

    /// Publish hosted feature services
    PortalPublisherPublishFeatures,

    /// Publish scene layers
    PortalPublisherPublishScenes,

    /// Publish server GP services
    PortalPublisherPublishServerGPServices,

    /// Publish tiled imagery layers
    PortalPublisherPublishTiledImagery,

    /// Publish tile layers
    PortalPublisherPublishTiles,

    // ============================================================================
    // Portal - User Privileges (17 privileges)
    // ============================================================================
    /// Add external members to groups
    PortalUserAddExternalMembersToGroup,

    /// Create groups
    PortalUserCreateGroup,

    /// Create items
    PortalUserCreateItem,

    /// Create workflows
    PortalUserCreateWorkflow,

    /// Invite partnered collaboration members
    PortalUserInvitePartneredCollaborationMembers,

    /// Join groups
    PortalUserJoinGroup,

    /// Join external (non-organizational) groups
    PortalUserJoinNonOrgGroup,

    /// Reassign items
    PortalUserReassignItems,

    /// Share groups with organization
    PortalUserShareGroupToOrg,

    /// Share groups publicly
    PortalUserShareGroupToPublic,

    /// Share content with groups
    PortalUserShareToGroup,

    /// Share OWN content with organization
    PortalUserShareToOrg,

    /// Share OWN content publicly
    PortalUserShareToPublic,

    /// View organizational groups
    PortalUserViewOrgGroups,

    /// View organizational items
    PortalUserViewOrgItems,

    /// View organizational users
    PortalUserViewOrgUsers,

    /// View tracks
    PortalUserViewTracks,

    // ============================================================================
    // Premium - Publisher Privileges (4 privileges)
    // ============================================================================
    /// Create advanced notebooks
    PremiumPublisherCreateAdvancedNotebooks,

    /// Create notebooks
    PremiumPublisherCreateNotebooks,

    /// Perform raster analysis
    PremiumPublisherRasteranalysis,

    /// Schedule notebooks
    PremiumPublisherScheduleNotebooks,

    // ============================================================================
    // Premium - User Privileges (18 privileges)
    // ============================================================================
    /// Access basemap services
    PremiumUserBasemaps,

    /// Access demographic data
    PremiumUserDemographics,

    /// Generate feature reports
    PremiumUserFeaturereport,

    /// Use geocoding services
    PremiumUserGeocode,

    /// Use stored geocoding
    PremiumUserGeocodeStored,

    /// Use temporary geocoding
    PremiumUserGeocodeTemporary,

    /// Access geoenrichment data
    PremiumUserGeoenrichment,

    /// Use network analysis services
    PremiumUserNetworkanalysis,

    /// Use closest facility analysis
    PremiumUserNetworkanalysisClosestfacility,

    /// Use last mile delivery analysis
    PremiumUserNetworkanalysisLastmiledelivery,

    /// Use location-allocation analysis
    PremiumUserNetworkanalysisLocationallocation,

    /// Use optimized routing
    PremiumUserNetworkanalysisOptimizedrouting,

    /// Use origin-destination cost matrix
    PremiumUserNetworkanalysisOrigindestinationcostmatrix,

    /// Use routing services
    PremiumUserNetworkanalysisRouting,

    /// Use service area analysis
    PremiumUserNetworkanalysisServicearea,

    /// Use snap-to-roads
    PremiumUserNetworkanalysisSnaptoroads,

    /// Use vehicle routing
    PremiumUserNetworkanalysisVehiclerouting,

    /// Use spatial analysis tools
    PremiumUserSpatialanalysis,

    // ============================================================================
    // OpenData - User Privileges (1 privilege)
    // ============================================================================
    /// Designate groups for Open Data
    OpendataUserDesignateGroup,
}

impl Permission {
    /// Convert permission to ESRI privilege string.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::Permission;
    ///
    /// let perm = Permission::PortalUserCreateItem;
    /// assert_eq!(perm.to_esri_string(), "portal:user:createItem");
    /// ```
    pub fn to_esri_string(&self) -> &'static str {
        match self {
            // Features
            Self::FeaturesUserEdit => "features:user:edit",
            Self::FeaturesUserFullEdit => "features:user:fullEdit",

            // Portal - Admin
            Self::PortalAdminAssignToGroups => "portal:admin:assignToGroups",
            Self::PortalAdminChangeUserRoles => "portal:admin:changeUserRoles",
            Self::PortalAdminCreateLeavingDisallowedGroup => {
                "portal:admin:createLeavingDisallowedGroup"
            }
            Self::PortalAdminCreateReports => "portal:admin:createReports",
            Self::PortalAdminCreateUpdateCapableGroup => "portal:admin:createUpdateCapableGroup",
            Self::PortalAdminDeleteGroups => "portal:admin:deleteGroups",
            Self::PortalAdminDeleteItems => "portal:admin:deleteItems",
            Self::PortalAdminDeleteUsers => "portal:admin:deleteUsers",
            Self::PortalAdminDisableUsers => "portal:admin:disableUsers",
            Self::PortalAdminInviteUsers => "portal:admin:inviteUsers",
            Self::PortalAdminManageCollaborations => "portal:admin:manageCollaborations",
            Self::PortalAdminManageCredits => "portal:admin:manageCredits",
            Self::PortalAdminManageEnterpriseGroups => "portal:admin:manageEnterpriseGroups",
            Self::PortalAdminManageLicenses => "portal:admin:manageLicenses",
            Self::PortalAdminManageRoles => "portal:admin:manageRoles",
            Self::PortalAdminManageSecurity => "portal:admin:manageSecurity",
            Self::PortalAdminManageUtilityServices => "portal:admin:manageUtilityServices",
            Self::PortalAdminManageWebsite => "portal:admin:manageWebsite",
            Self::PortalAdminReassignGroups => "portal:admin:reassignGroups",
            Self::PortalAdminReassignItems => "portal:admin:reassignItems",
            Self::PortalAdminShareToOrg => "portal:admin:shareToOrg",
            Self::PortalAdminShareToPublic => "portal:admin:shareToPublic",
            Self::PortalAdminUpdateGroups => "portal:admin:updateGroups",
            Self::PortalAdminUpdateItemCategorySchema => "portal:admin:updateItemCategorySchema",
            Self::PortalAdminUpdateItems => "portal:admin:updateItems",
            Self::PortalAdminUpdateMemberCategorySchema => {
                "portal:admin:updateMemberCategorySchema"
            }
            Self::PortalAdminUpdateUsers => "portal:admin:updateUsers",
            Self::PortalAdminViewGroups => "portal:admin:viewGroups",
            Self::PortalAdminViewItems => "portal:admin:viewItems",
            Self::PortalAdminViewUsers => "portal:admin:viewUsers",

            // Portal - Publisher
            Self::PortalPublisherCreateDataPipelines => "portal:publisher:createDataPipelines",
            Self::PortalPublisherPublishDynamicImagery => "portal:publisher:publishDynamicImagery",
            Self::PortalPublisherPublishFeatures => "portal:publisher:publishFeatures",
            Self::PortalPublisherPublishScenes => "portal:publisher:publishScenes",
            Self::PortalPublisherPublishServerGPServices => {
                "portal:publisher:publishServerGPServices"
            }
            Self::PortalPublisherPublishTiledImagery => "portal:publisher:publishTiledImagery",
            Self::PortalPublisherPublishTiles => "portal:publisher:publishTiles",

            // Portal - User
            Self::PortalUserAddExternalMembersToGroup => "portal:user:addExternalMembersToGroup",
            Self::PortalUserCreateGroup => "portal:user:createGroup",
            Self::PortalUserCreateItem => "portal:user:createItem",
            Self::PortalUserCreateWorkflow => "portal:user:createWorkflow",
            Self::PortalUserInvitePartneredCollaborationMembers => {
                "portal:user:invitePartneredCollaborationMembers"
            }
            Self::PortalUserJoinGroup => "portal:user:joinGroup",
            Self::PortalUserJoinNonOrgGroup => "portal:user:joinNonOrgGroup",
            Self::PortalUserReassignItems => "portal:user:reassignItems",
            Self::PortalUserShareGroupToOrg => "portal:user:shareGroupToOrg",
            Self::PortalUserShareGroupToPublic => "portal:user:shareGroupToPublic",
            Self::PortalUserShareToGroup => "portal:user:shareToGroup",
            Self::PortalUserShareToOrg => "portal:user:shareToOrg",
            Self::PortalUserShareToPublic => "portal:user:shareToPublic",
            Self::PortalUserViewOrgGroups => "portal:user:viewOrgGroups",
            Self::PortalUserViewOrgItems => "portal:user:viewOrgItems",
            Self::PortalUserViewOrgUsers => "portal:user:viewOrgUsers",
            Self::PortalUserViewTracks => "portal:user:viewTracks",

            // Premium - Publisher
            Self::PremiumPublisherCreateAdvancedNotebooks => {
                "premium:publisher:createAdvancedNotebooks"
            }
            Self::PremiumPublisherCreateNotebooks => "premium:publisher:createNotebooks",
            Self::PremiumPublisherRasteranalysis => "premium:publisher:rasteranalysis",
            Self::PremiumPublisherScheduleNotebooks => "premium:publisher:scheduleNotebooks",

            // Premium - User
            Self::PremiumUserBasemaps => "premium:user:basemaps",
            Self::PremiumUserDemographics => "premium:user:demographics",
            Self::PremiumUserFeaturereport => "premium:user:featurereport",
            Self::PremiumUserGeocode => "premium:user:geocode",
            Self::PremiumUserGeocodeStored => "premium:user:geocode:stored",
            Self::PremiumUserGeocodeTemporary => "premium:user:geocode:temporary",
            Self::PremiumUserGeoenrichment => "premium:user:geoenrichment",
            Self::PremiumUserNetworkanalysis => "premium:user:networkanalysis",
            Self::PremiumUserNetworkanalysisClosestfacility => {
                "premium:user:networkanalysis:closestfacility"
            }
            Self::PremiumUserNetworkanalysisLastmiledelivery => {
                "premium:user:networkanalysis:lastmiledelivery"
            }
            Self::PremiumUserNetworkanalysisLocationallocation => {
                "premium:user:networkanalysis:locationallocation"
            }
            Self::PremiumUserNetworkanalysisOptimizedrouting => {
                "premium:user:networkanalysis:optimizedrouting"
            }
            Self::PremiumUserNetworkanalysisOrigindestinationcostmatrix => {
                "premium:user:networkanalysis:origindestinationcostmatrix"
            }
            Self::PremiumUserNetworkanalysisRouting => "premium:user:networkanalysis:routing",
            Self::PremiumUserNetworkanalysisServicearea => {
                "premium:user:networkanalysis:servicearea"
            }
            Self::PremiumUserNetworkanalysisSnaptoroads => {
                "premium:user:networkanalysis:snaptoroads"
            }
            Self::PremiumUserNetworkanalysisVehiclerouting => {
                "premium:user:networkanalysis:vehiclerouting"
            }
            Self::PremiumUserSpatialanalysis => "premium:user:spatialanalysis",

            // OpenData
            Self::OpendataUserDesignateGroup => "opendata:user:designateGroup",
        }
    }

    /// Parse ESRI privilege string into Permission enum.
    ///
    /// Returns `None` if the string doesn't match any known permission.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::Permission;
    ///
    /// let perm = Permission::from_esri_string("portal:user:createItem");
    /// assert_eq!(perm, Some(Permission::PortalUserCreateItem));
    ///
    /// let unknown = Permission::from_esri_string("unknown:permission");
    /// assert_eq!(unknown, None);
    /// ```
    pub fn from_esri_string(s: &str) -> Option<Self> {
        match s {
            // Features
            "features:user:edit" => Some(Self::FeaturesUserEdit),
            "features:user:fullEdit" => Some(Self::FeaturesUserFullEdit),

            // Portal - Admin
            "portal:admin:assignToGroups" => Some(Self::PortalAdminAssignToGroups),
            "portal:admin:changeUserRoles" => Some(Self::PortalAdminChangeUserRoles),
            "portal:admin:createLeavingDisallowedGroup" => {
                Some(Self::PortalAdminCreateLeavingDisallowedGroup)
            }
            "portal:admin:createReports" => Some(Self::PortalAdminCreateReports),
            "portal:admin:createUpdateCapableGroup" => {
                Some(Self::PortalAdminCreateUpdateCapableGroup)
            }
            "portal:admin:deleteGroups" => Some(Self::PortalAdminDeleteGroups),
            "portal:admin:deleteItems" => Some(Self::PortalAdminDeleteItems),
            "portal:admin:deleteUsers" => Some(Self::PortalAdminDeleteUsers),
            "portal:admin:disableUsers" => Some(Self::PortalAdminDisableUsers),
            "portal:admin:inviteUsers" => Some(Self::PortalAdminInviteUsers),
            "portal:admin:manageCollaborations" => Some(Self::PortalAdminManageCollaborations),
            "portal:admin:manageCredits" => Some(Self::PortalAdminManageCredits),
            "portal:admin:manageEnterpriseGroups" => Some(Self::PortalAdminManageEnterpriseGroups),
            "portal:admin:manageLicenses" => Some(Self::PortalAdminManageLicenses),
            "portal:admin:manageRoles" => Some(Self::PortalAdminManageRoles),
            "portal:admin:manageSecurity" => Some(Self::PortalAdminManageSecurity),
            "portal:admin:manageUtilityServices" => Some(Self::PortalAdminManageUtilityServices),
            "portal:admin:manageWebsite" => Some(Self::PortalAdminManageWebsite),
            "portal:admin:reassignGroups" => Some(Self::PortalAdminReassignGroups),
            "portal:admin:reassignItems" => Some(Self::PortalAdminReassignItems),
            "portal:admin:shareToOrg" => Some(Self::PortalAdminShareToOrg),
            "portal:admin:shareToPublic" => Some(Self::PortalAdminShareToPublic),
            "portal:admin:updateGroups" => Some(Self::PortalAdminUpdateGroups),
            "portal:admin:updateItemCategorySchema" => {
                Some(Self::PortalAdminUpdateItemCategorySchema)
            }
            "portal:admin:updateItems" => Some(Self::PortalAdminUpdateItems),
            "portal:admin:updateMemberCategorySchema" => {
                Some(Self::PortalAdminUpdateMemberCategorySchema)
            }
            "portal:admin:updateUsers" => Some(Self::PortalAdminUpdateUsers),
            "portal:admin:viewGroups" => Some(Self::PortalAdminViewGroups),
            "portal:admin:viewItems" => Some(Self::PortalAdminViewItems),
            "portal:admin:viewUsers" => Some(Self::PortalAdminViewUsers),

            // Portal - Publisher
            "portal:publisher:createDataPipelines" => {
                Some(Self::PortalPublisherCreateDataPipelines)
            }
            "portal:publisher:publishDynamicImagery" => {
                Some(Self::PortalPublisherPublishDynamicImagery)
            }
            "portal:publisher:publishFeatures" => Some(Self::PortalPublisherPublishFeatures),
            "portal:publisher:publishScenes" => Some(Self::PortalPublisherPublishScenes),
            "portal:publisher:publishServerGPServices" => {
                Some(Self::PortalPublisherPublishServerGPServices)
            }
            "portal:publisher:publishTiledImagery" => {
                Some(Self::PortalPublisherPublishTiledImagery)
            }
            "portal:publisher:publishTiles" => Some(Self::PortalPublisherPublishTiles),

            // Portal - User
            "portal:user:addExternalMembersToGroup" => {
                Some(Self::PortalUserAddExternalMembersToGroup)
            }
            "portal:user:createGroup" => Some(Self::PortalUserCreateGroup),
            "portal:user:createItem" => Some(Self::PortalUserCreateItem),
            "portal:user:createWorkflow" => Some(Self::PortalUserCreateWorkflow),
            "portal:user:invitePartneredCollaborationMembers" => {
                Some(Self::PortalUserInvitePartneredCollaborationMembers)
            }
            "portal:user:joinGroup" => Some(Self::PortalUserJoinGroup),
            "portal:user:joinNonOrgGroup" => Some(Self::PortalUserJoinNonOrgGroup),
            "portal:user:reassignItems" => Some(Self::PortalUserReassignItems),
            "portal:user:shareGroupToOrg" => Some(Self::PortalUserShareGroupToOrg),
            "portal:user:shareGroupToPublic" => Some(Self::PortalUserShareGroupToPublic),
            "portal:user:shareToGroup" => Some(Self::PortalUserShareToGroup),
            "portal:user:shareToOrg" => Some(Self::PortalUserShareToOrg),
            "portal:user:shareToPublic" => Some(Self::PortalUserShareToPublic),
            "portal:user:viewOrgGroups" => Some(Self::PortalUserViewOrgGroups),
            "portal:user:viewOrgItems" => Some(Self::PortalUserViewOrgItems),
            "portal:user:viewOrgUsers" => Some(Self::PortalUserViewOrgUsers),
            "portal:user:viewTracks" => Some(Self::PortalUserViewTracks),

            // Premium - Publisher
            "premium:publisher:createAdvancedNotebooks" => {
                Some(Self::PremiumPublisherCreateAdvancedNotebooks)
            }
            "premium:publisher:createNotebooks" => Some(Self::PremiumPublisherCreateNotebooks),
            "premium:publisher:rasteranalysis" => Some(Self::PremiumPublisherRasteranalysis),
            "premium:publisher:scheduleNotebooks" => Some(Self::PremiumPublisherScheduleNotebooks),

            // Premium - User
            "premium:user:basemaps" => Some(Self::PremiumUserBasemaps),
            "premium:user:demographics" => Some(Self::PremiumUserDemographics),
            "premium:user:featurereport" => Some(Self::PremiumUserFeaturereport),
            "premium:user:geocode" => Some(Self::PremiumUserGeocode),
            "premium:user:geocode:stored" => Some(Self::PremiumUserGeocodeStored),
            "premium:user:geocode:temporary" => Some(Self::PremiumUserGeocodeTemporary),
            "premium:user:geoenrichment" => Some(Self::PremiumUserGeoenrichment),
            "premium:user:networkanalysis" => Some(Self::PremiumUserNetworkanalysis),
            "premium:user:networkanalysis:closestfacility" => {
                Some(Self::PremiumUserNetworkanalysisClosestfacility)
            }
            "premium:user:networkanalysis:lastmiledelivery" => {
                Some(Self::PremiumUserNetworkanalysisLastmiledelivery)
            }
            "premium:user:networkanalysis:locationallocation" => {
                Some(Self::PremiumUserNetworkanalysisLocationallocation)
            }
            "premium:user:networkanalysis:optimizedrouting" => {
                Some(Self::PremiumUserNetworkanalysisOptimizedrouting)
            }
            "premium:user:networkanalysis:origindestinationcostmatrix" => {
                Some(Self::PremiumUserNetworkanalysisOrigindestinationcostmatrix)
            }
            "premium:user:networkanalysis:routing" => {
                Some(Self::PremiumUserNetworkanalysisRouting)
            }
            "premium:user:networkanalysis:servicearea" => {
                Some(Self::PremiumUserNetworkanalysisServicearea)
            }
            "premium:user:networkanalysis:snaptoroads" => {
                Some(Self::PremiumUserNetworkanalysisSnaptoroads)
            }
            "premium:user:networkanalysis:vehiclerouting" => {
                Some(Self::PremiumUserNetworkanalysisVehiclerouting)
            }
            "premium:user:spatialanalysis" => Some(Self::PremiumUserSpatialanalysis),

            // OpenData
            "opendata:user:designateGroup" => Some(Self::OpendataUserDesignateGroup),

            _ => None,
        }
    }

    /// Human-readable description of what this permission allows.
    ///
    /// Used in error messages and documentation.
    pub fn description(&self) -> &'static str {
        match self {
            // Features
            Self::FeaturesUserEdit => "Edit features (respects layer permissions)",
            Self::FeaturesUserFullEdit => "Full edit capability (bypasses layer restrictions)",

            // Portal - Admin
            Self::PortalAdminAssignToGroups => "Assign members to groups",
            Self::PortalAdminChangeUserRoles => "Change user roles",
            Self::PortalAdminCreateLeavingDisallowedGroup => {
                "Create groups that members cannot leave"
            }
            Self::PortalAdminCreateReports => "Create organization reports",
            Self::PortalAdminCreateUpdateCapableGroup => "Create update-capable groups",
            Self::PortalAdminDeleteGroups => "Delete groups",
            Self::PortalAdminDeleteItems => "Delete items",
            Self::PortalAdminDeleteUsers => "Delete users",
            Self::PortalAdminDisableUsers => "Disable user accounts",
            Self::PortalAdminInviteUsers => "Invite users to organization",
            Self::PortalAdminManageCollaborations => "Manage collaboration agreements",
            Self::PortalAdminManageCredits => "Manage organization credits",
            Self::PortalAdminManageEnterpriseGroups => "Manage enterprise groups",
            Self::PortalAdminManageLicenses => "Manage user licenses",
            Self::PortalAdminManageRoles => "Create and modify custom roles",
            Self::PortalAdminManageSecurity => "Manage organization security settings",
            Self::PortalAdminManageUtilityServices => "Manage utility services",
            Self::PortalAdminManageWebsite => "Manage organization website",
            Self::PortalAdminReassignGroups => "Reassign group ownership",
            Self::PortalAdminReassignItems => "Reassign item ownership",
            Self::PortalAdminShareToOrg => "Share other users' content with organization",
            Self::PortalAdminShareToPublic => "Share other users' content publicly",
            Self::PortalAdminUpdateGroups => "Update group information",
            Self::PortalAdminUpdateItemCategorySchema => "Update item category schema",
            Self::PortalAdminUpdateItems => "Update items",
            Self::PortalAdminUpdateMemberCategorySchema => "Update member category schema",
            Self::PortalAdminUpdateUsers => "Update user accounts",
            Self::PortalAdminViewGroups => "View all groups (including private)",
            Self::PortalAdminViewItems => "View all items",
            Self::PortalAdminViewUsers => "View all users",

            // Portal - Publisher
            Self::PortalPublisherCreateDataPipelines => "Create data pipelines",
            Self::PortalPublisherPublishDynamicImagery => "Publish dynamic imagery layers",
            Self::PortalPublisherPublishFeatures => "Publish hosted feature services",
            Self::PortalPublisherPublishScenes => "Publish scene layers",
            Self::PortalPublisherPublishServerGPServices => "Publish server GP services",
            Self::PortalPublisherPublishTiledImagery => "Publish tiled imagery layers",
            Self::PortalPublisherPublishTiles => "Publish tile layers",

            // Portal - User
            Self::PortalUserAddExternalMembersToGroup => "Add external members to groups",
            Self::PortalUserCreateGroup => "Create groups",
            Self::PortalUserCreateItem => "Create, update, delete own content",
            Self::PortalUserCreateWorkflow => "Create workflows",
            Self::PortalUserInvitePartneredCollaborationMembers => {
                "Invite partnered collaboration members"
            }
            Self::PortalUserJoinGroup => "Join groups",
            Self::PortalUserJoinNonOrgGroup => "Join external (non-organizational) groups",
            Self::PortalUserReassignItems => "Reassign items",
            Self::PortalUserShareGroupToOrg => "Share groups with organization",
            Self::PortalUserShareGroupToPublic => "Share groups publicly",
            Self::PortalUserShareToGroup => "Share content with groups",
            Self::PortalUserShareToOrg => "Share own content with organization",
            Self::PortalUserShareToPublic => "Share own content publicly",
            Self::PortalUserViewOrgGroups => "View organizational groups",
            Self::PortalUserViewOrgItems => "View organizational items",
            Self::PortalUserViewOrgUsers => "View organizational users",
            Self::PortalUserViewTracks => "View tracks",

            // Premium - Publisher
            Self::PremiumPublisherCreateAdvancedNotebooks => "Create advanced notebooks",
            Self::PremiumPublisherCreateNotebooks => "Create notebooks",
            Self::PremiumPublisherRasteranalysis => "Perform raster analysis",
            Self::PremiumPublisherScheduleNotebooks => "Schedule notebooks",

            // Premium - User
            Self::PremiumUserBasemaps => "Access basemap services",
            Self::PremiumUserDemographics => "Access demographic data",
            Self::PremiumUserFeaturereport => "Generate feature reports",
            Self::PremiumUserGeocode => "Use geocoding services",
            Self::PremiumUserGeocodeStored => "Use stored geocoding",
            Self::PremiumUserGeocodeTemporary => "Use temporary geocoding",
            Self::PremiumUserGeoenrichment => "Access geoenrichment data",
            Self::PremiumUserNetworkanalysis => "Use network analysis services",
            Self::PremiumUserNetworkanalysisClosestfacility => "Use closest facility analysis",
            Self::PremiumUserNetworkanalysisLastmiledelivery => "Use last mile delivery analysis",
            Self::PremiumUserNetworkanalysisLocationallocation => {
                "Use location-allocation analysis"
            }
            Self::PremiumUserNetworkanalysisOptimizedrouting => "Use optimized routing",
            Self::PremiumUserNetworkanalysisOrigindestinationcostmatrix => {
                "Use origin-destination cost matrix"
            }
            Self::PremiumUserNetworkanalysisRouting => "Use routing services",
            Self::PremiumUserNetworkanalysisServicearea => "Use service area analysis",
            Self::PremiumUserNetworkanalysisSnaptoroads => "Use snap-to-roads",
            Self::PremiumUserNetworkanalysisVehiclerouting => "Use vehicle routing",
            Self::PremiumUserSpatialanalysis => "Use spatial analysis tools",

            // OpenData
            Self::OpendataUserDesignateGroup => "Designate groups for Open Data",
        }
    }

    /// Default tier/group this permission typically belongs to.
    ///
    /// Used for fallback routing when no specific permission key is configured.
    pub fn default_tier(&self) -> ApiKeyTier {
        match self {
            // Features belong to General tier
            Self::FeaturesUserEdit | Self::FeaturesUserFullEdit => ApiKeyTier::General,

            // Portal Admin permissions
            Self::PortalAdminAssignToGroups
            | Self::PortalAdminChangeUserRoles
            | Self::PortalAdminCreateLeavingDisallowedGroup
            | Self::PortalAdminCreateReports
            | Self::PortalAdminCreateUpdateCapableGroup
            | Self::PortalAdminDeleteGroups
            | Self::PortalAdminDeleteItems
            | Self::PortalAdminDeleteUsers
            | Self::PortalAdminDisableUsers
            | Self::PortalAdminInviteUsers
            | Self::PortalAdminManageCollaborations
            | Self::PortalAdminManageCredits
            | Self::PortalAdminManageEnterpriseGroups
            | Self::PortalAdminManageLicenses
            | Self::PortalAdminManageRoles
            | Self::PortalAdminManageSecurity
            | Self::PortalAdminManageUtilityServices
            | Self::PortalAdminManageWebsite
            | Self::PortalAdminReassignGroups
            | Self::PortalAdminReassignItems
            | Self::PortalAdminShareToOrg
            | Self::PortalAdminShareToPublic
            | Self::PortalAdminUpdateGroups
            | Self::PortalAdminUpdateItemCategorySchema
            | Self::PortalAdminUpdateItems
            | Self::PortalAdminUpdateMemberCategorySchema
            | Self::PortalAdminUpdateUsers
            | Self::PortalAdminViewGroups
            | Self::PortalAdminViewItems
            | Self::PortalAdminViewUsers => ApiKeyTier::Admin,

            // Portal Publisher and User permissions belong to General
            Self::PortalPublisherCreateDataPipelines
            | Self::PortalPublisherPublishDynamicImagery
            | Self::PortalPublisherPublishFeatures
            | Self::PortalPublisherPublishScenes
            | Self::PortalPublisherPublishServerGPServices
            | Self::PortalPublisherPublishTiledImagery
            | Self::PortalPublisherPublishTiles
            | Self::PortalUserAddExternalMembersToGroup
            | Self::PortalUserCreateGroup
            | Self::PortalUserCreateItem
            | Self::PortalUserCreateWorkflow
            | Self::PortalUserInvitePartneredCollaborationMembers
            | Self::PortalUserJoinGroup
            | Self::PortalUserJoinNonOrgGroup
            | Self::PortalUserReassignItems
            | Self::PortalUserShareGroupToOrg
            | Self::PortalUserShareGroupToPublic
            | Self::PortalUserShareToGroup
            | Self::PortalUserShareToOrg
            | Self::PortalUserShareToPublic
            | Self::PortalUserViewOrgGroups
            | Self::PortalUserViewOrgItems
            | Self::PortalUserViewOrgUsers
            | Self::PortalUserViewTracks
            | Self::PremiumPublisherCreateAdvancedNotebooks
            | Self::PremiumPublisherCreateNotebooks
            | Self::PremiumPublisherScheduleNotebooks => ApiKeyTier::General,

            // Premium User - Location services
            Self::PremiumUserGeocode
            | Self::PremiumUserGeocodeStored
            | Self::PremiumUserGeocodeTemporary
            | Self::PremiumUserGeoenrichment
            | Self::PremiumUserNetworkanalysis
            | Self::PremiumUserNetworkanalysisClosestfacility
            | Self::PremiumUserNetworkanalysisLastmiledelivery
            | Self::PremiumUserNetworkanalysisLocationallocation
            | Self::PremiumUserNetworkanalysisOptimizedrouting
            | Self::PremiumUserNetworkanalysisOrigindestinationcostmatrix
            | Self::PremiumUserNetworkanalysisRouting
            | Self::PremiumUserNetworkanalysisServicearea
            | Self::PremiumUserNetworkanalysisSnaptoroads
            | Self::PremiumUserNetworkanalysisVehiclerouting => ApiKeyTier::Location,

            // Premium User - Spatial services
            Self::PremiumPublisherRasteranalysis | Self::PremiumUserSpatialanalysis => {
                ApiKeyTier::Spatial
            }

            // Premium User - Demographics and general
            Self::PremiumUserDemographics | Self::PremiumUserFeaturereport => ApiKeyTier::General,

            // Basemaps are public
            Self::PremiumUserBasemaps => ApiKeyTier::Public,

            // OpenData belongs to General
            Self::OpendataUserDesignateGroup => ApiKeyTier::General,
        }
    }

    /// Methods that require this permission (reverse lookup).
    ///
    /// Used for error messages, testing, and documentation generation.
    /// Returns fully-qualified method names like `"PortalClient::delete_item"`.
    ///
    /// # Note
    ///
    /// This mapping must be kept in sync with actual method implementations.
    /// Tests should verify that all listed methods actually exist.
    pub fn methods(&self) -> &'static [&'static str] {
        match self {
            // Portal - User permissions
            // Note: createItem permission covers create, update, AND delete operations on own content
            Self::PortalUserCreateItem => &[
                "PortalClient::add_item",
                "PortalClient::update_item",
                "PortalClient::delete_item",
                "PortalClient::update_item_data_v2",
                "PortalClient::delete_service",
            ],

            Self::PortalUserCreateGroup => &[
                "PortalClient::create_group",
                "PortalClient::update_group",
                "PortalClient::delete_group",
            ],

            Self::PortalUserJoinGroup => &[
                "PortalClient::join_group",
                "PortalClient::leave_group",
            ],

            Self::PortalUserShareToGroup => &["PortalClient::share_item"],
            Self::PortalUserShareToOrg => &["PortalClient::share_item"],
            Self::PortalUserShareToPublic => &["PortalClient::share_item"],

            // Portal - Publisher permissions
            Self::PortalPublisherPublishFeatures => &[
                "PortalClient::create_service",
                "PortalClient::publish",
                "PortalClient::update_service_definition",
                "PortalClient::add_to_definition",
                "PortalClient::add_to_definition_with_url",
                "PortalClient::overwrite_service",
            ],

            // Portal - Admin permissions
            Self::PortalAdminAssignToGroups => &[
                "PortalClient::add_to_group",
                "PortalClient::remove_from_group",
            ],

            Self::PortalAdminUpdateGroups => &[
                "PortalClient::update_group",
                "PortalClient::delete_group",
            ],

            Self::PortalAdminDeleteGroups => &["PortalClient::delete_group"],

            Self::PortalAdminUpdateItems => &[
                "PortalClient::update_item",
                "PortalClient::update_service_definition",
            ],

            Self::PortalAdminDeleteItems => &["PortalClient::delete_item"],

            // Read-only operations (no permissions required beyond authentication)
            // get_item, get_item_data, get_self, search, search_groups, get_group, etc.
            // These don't need permission annotations

            // Premium and other permissions - populate as needed
            _ => &[],
        }
    }

    /// Find which permission is needed for a method (helper for error enhancement).
    ///
    /// Searches all permissions for one whose `methods()` list contains the given method name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(perm) = Permission::find_for_method("PortalClient::delete_item") {
    ///     println!("Requires: {}", perm.to_esri_string());
    /// }
    /// ```
    pub fn find_for_method(full_method_name: &str) -> Option<Self> {
        ALL_PERMISSIONS
            .iter()
            .find(|perm| perm.methods().contains(&full_method_name))
            .copied()
    }
}

/// All permission variants for iteration.
///
/// Used by `find_for_method()` and testing.
const ALL_PERMISSIONS: &[Permission] = &[
    // Features
    Permission::FeaturesUserEdit,
    Permission::FeaturesUserFullEdit,
    // Portal - Admin
    Permission::PortalAdminAssignToGroups,
    Permission::PortalAdminChangeUserRoles,
    Permission::PortalAdminCreateLeavingDisallowedGroup,
    Permission::PortalAdminCreateReports,
    Permission::PortalAdminCreateUpdateCapableGroup,
    Permission::PortalAdminDeleteGroups,
    Permission::PortalAdminDeleteItems,
    Permission::PortalAdminDeleteUsers,
    Permission::PortalAdminDisableUsers,
    Permission::PortalAdminInviteUsers,
    Permission::PortalAdminManageCollaborations,
    Permission::PortalAdminManageCredits,
    Permission::PortalAdminManageEnterpriseGroups,
    Permission::PortalAdminManageLicenses,
    Permission::PortalAdminManageRoles,
    Permission::PortalAdminManageSecurity,
    Permission::PortalAdminManageUtilityServices,
    Permission::PortalAdminManageWebsite,
    Permission::PortalAdminReassignGroups,
    Permission::PortalAdminReassignItems,
    Permission::PortalAdminShareToOrg,
    Permission::PortalAdminShareToPublic,
    Permission::PortalAdminUpdateGroups,
    Permission::PortalAdminUpdateItemCategorySchema,
    Permission::PortalAdminUpdateItems,
    Permission::PortalAdminUpdateMemberCategorySchema,
    Permission::PortalAdminUpdateUsers,
    Permission::PortalAdminViewGroups,
    Permission::PortalAdminViewItems,
    Permission::PortalAdminViewUsers,
    // Portal - Publisher
    Permission::PortalPublisherCreateDataPipelines,
    Permission::PortalPublisherPublishDynamicImagery,
    Permission::PortalPublisherPublishFeatures,
    Permission::PortalPublisherPublishScenes,
    Permission::PortalPublisherPublishServerGPServices,
    Permission::PortalPublisherPublishTiledImagery,
    Permission::PortalPublisherPublishTiles,
    // Portal - User
    Permission::PortalUserAddExternalMembersToGroup,
    Permission::PortalUserCreateGroup,
    Permission::PortalUserCreateItem,
    Permission::PortalUserCreateWorkflow,
    Permission::PortalUserInvitePartneredCollaborationMembers,
    Permission::PortalUserJoinGroup,
    Permission::PortalUserJoinNonOrgGroup,
    Permission::PortalUserReassignItems,
    Permission::PortalUserShareGroupToOrg,
    Permission::PortalUserShareGroupToPublic,
    Permission::PortalUserShareToGroup,
    Permission::PortalUserShareToOrg,
    Permission::PortalUserShareToPublic,
    Permission::PortalUserViewOrgGroups,
    Permission::PortalUserViewOrgItems,
    Permission::PortalUserViewOrgUsers,
    Permission::PortalUserViewTracks,
    // Premium - Publisher
    Permission::PremiumPublisherCreateAdvancedNotebooks,
    Permission::PremiumPublisherCreateNotebooks,
    Permission::PremiumPublisherRasteranalysis,
    Permission::PremiumPublisherScheduleNotebooks,
    // Premium - User
    Permission::PremiumUserBasemaps,
    Permission::PremiumUserDemographics,
    Permission::PremiumUserFeaturereport,
    Permission::PremiumUserGeocode,
    Permission::PremiumUserGeocodeStored,
    Permission::PremiumUserGeocodeTemporary,
    Permission::PremiumUserGeoenrichment,
    Permission::PremiumUserNetworkanalysis,
    Permission::PremiumUserNetworkanalysisClosestfacility,
    Permission::PremiumUserNetworkanalysisLastmiledelivery,
    Permission::PremiumUserNetworkanalysisLocationallocation,
    Permission::PremiumUserNetworkanalysisOptimizedrouting,
    Permission::PremiumUserNetworkanalysisOrigindestinationcostmatrix,
    Permission::PremiumUserNetworkanalysisRouting,
    Permission::PremiumUserNetworkanalysisServicearea,
    Permission::PremiumUserNetworkanalysisSnaptoroads,
    Permission::PremiumUserNetworkanalysisVehiclerouting,
    Permission::PremiumUserSpatialanalysis,
    // OpenData
    Permission::OpendataUserDesignateGroup,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_string_round_trip() {
        for perm in ALL_PERMISSIONS {
            let esri_str = perm.to_esri_string();
            let parsed = Permission::from_esri_string(esri_str);
            assert_eq!(
                Some(*perm),
                parsed,
                "Round-trip failed for {:?}",
                perm
            );
        }
    }

    #[test]
    fn test_all_permissions_have_descriptions() {
        for perm in ALL_PERMISSIONS {
            let desc = perm.description();
            assert!(
                !desc.is_empty(),
                "Permission {:?} has no description",
                perm
            );
        }
    }

    #[test]
    fn test_all_permissions_have_default_tier() {
        for perm in ALL_PERMISSIONS {
            // Just ensure it doesn't panic
            let _ = perm.default_tier();
        }
    }
}
