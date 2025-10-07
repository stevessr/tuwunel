mod watch;

use std::{
	collections::{BTreeMap, BTreeSet},
	sync::{Arc, Mutex, Mutex as StdMutex},
};

use ruma::{
	DeviceId, OwnedDeviceId, OwnedRoomId, OwnedUserId, UserId,
	api::client::sync::sync_events::v5::{
		Request, request,
		request::{AccountData, E2EE, Receipts, ToDevice, Typing},
	},
};
use tuwunel_core::{Result, err, implement, is_equal_to, smallstr::SmallString};
use tuwunel_database::Map;

pub struct Service {
	services: Arc<crate::services::OnceServices>,
	connections: Connections,
	db: Data,
}

pub struct Data {
	todeviceid_events: Arc<Map>,
	userroomid_joined: Arc<Map>,
	userroomid_invitestate: Arc<Map>,
	userroomid_leftstate: Arc<Map>,
	userroomid_knockedstate: Arc<Map>,
	userroomid_notificationcount: Arc<Map>,
	userroomid_highlightcount: Arc<Map>,
	pduid_pdu: Arc<Map>,
	keychangeid_userid: Arc<Map>,
	roomuserdataid_accountdata: Arc<Map>,
	roomusertype_roomuserdataid: Arc<Map>,
	readreceiptid_readreceipt: Arc<Map>,
	userid_lastonetimekeyupdate: Arc<Map>,
}

#[derive(Debug, Default)]
pub struct Cache {
	lists: Lists,
	known_rooms: KnownRooms,
	subscriptions: Subscriptions,
	extensions: request::Extensions,
}

type Connections = Mutex<BTreeMap<ConnectionKey, Connection>>;
type Connection = Arc<Mutex<Cache>>;
pub type ConnectionKey = (OwnedUserId, OwnedDeviceId, Option<ConnectionId>);
pub type ConnectionId = SmallString<[u8; 16]>;

pub type Subscriptions = BTreeMap<OwnedRoomId, request::RoomSubscription>;
pub type Lists = BTreeMap<ListId, request::List>;
pub type KnownRooms = BTreeMap<ListId, ListRooms>;
pub type ListRooms = BTreeMap<OwnedRoomId, u64>;
pub type ListId = SmallString<[u8; 16]>;

impl crate::Service for Service {
	fn build(args: &crate::Args<'_>) -> Result<Arc<Self>> {
		Ok(Arc::new(Self {
			db: Data {
				todeviceid_events: args.db["todeviceid_events"].clone(),
				userroomid_joined: args.db["userroomid_joined"].clone(),
				userroomid_invitestate: args.db["userroomid_invitestate"].clone(),
				userroomid_leftstate: args.db["userroomid_leftstate"].clone(),
				userroomid_knockedstate: args.db["userroomid_knockedstate"].clone(),
				userroomid_notificationcount: args.db["userroomid_notificationcount"].clone(),
				userroomid_highlightcount: args.db["userroomid_highlightcount"].clone(),
				pduid_pdu: args.db["pduid_pdu"].clone(),
				keychangeid_userid: args.db["keychangeid_userid"].clone(),
				roomuserdataid_accountdata: args.db["roomuserdataid_accountdata"].clone(),
				roomusertype_roomuserdataid: args.db["roomusertype_roomuserdataid"].clone(),
				readreceiptid_readreceipt: args.db["readreceiptid_readreceipt"].clone(),
				userid_lastonetimekeyupdate: args.db["userid_lastonetimekeyupdate"].clone(),
			},
			services: args.services.clone(),
			connections: StdMutex::new(BTreeMap::new()),
		}))
	}

	fn name(&self) -> &str { crate::service::make_name(std::module_path!()) }
}

#[implement(Service)]
pub fn update_cache(&self, key: &ConnectionKey, request: &mut Request) -> KnownRooms {
	let cache = self.get_connection(key);
	let mut cached = cache.lock().expect("locked");

	Self::update_cache_lists(request, &mut cached);
	Self::update_cache_subscriptions(request, &mut cached);
	Self::update_cache_extensions(request, &mut cached);

	cached.known_rooms.clone()
}

#[implement(Service)]
fn update_cache_lists(request: &mut Request, cached: &mut Cache) {
	for (list_id, request_list) in &mut request.lists {
		cached
			.lists
			.entry(list_id.as_str().into())
			.and_modify(|cached_list| {
				Self::update_cache_list(request_list, cached_list);
			})
			.or_insert_with(|| request_list.clone());
	}
}

#[implement(Service)]
fn update_cache_list(request: &mut request::List, cached: &mut request::List) {
	list_or_sticky(
		&mut request.room_details.required_state,
		&mut cached.room_details.required_state,
	);

	match (&mut request.filters, &mut cached.filters) {
		| (None, None) => {},
		| (None, Some(cached)) => request.filters = Some(cached.clone()),
		| (Some(request), None) => cached.filters = Some(request.clone()),
		| (Some(request), Some(cached)) => {
			some_or_sticky(&mut request.is_dm, &mut cached.is_dm);
			some_or_sticky(&mut request.is_encrypted, &mut cached.is_encrypted);
			some_or_sticky(&mut request.is_invite, &mut cached.is_invite);
			list_or_sticky(&mut request.room_types, &mut cached.room_types);
			list_or_sticky(&mut request.not_room_types, &mut cached.not_room_types);
			list_or_sticky(&mut request.tags, &mut cached.not_tags);
			list_or_sticky(&mut request.spaces, &mut cached.spaces);
		},
	}
}

#[implement(Service)]
fn update_cache_subscriptions(request: &mut Request, cached: &mut Cache) {
	cached
		.subscriptions
		.extend(request.room_subscriptions.clone());

	request
		.room_subscriptions
		.extend(cached.subscriptions.clone());
}

#[implement(Service)]
fn update_cache_extensions(request: &mut Request, cached: &mut Cache) {
	let request = &mut request.extensions;
	let cached = &mut cached.extensions;

	Self::update_cache_account_data(&mut request.account_data, &mut cached.account_data);
	Self::update_cache_receipts(&mut request.receipts, &mut cached.receipts);
	Self::update_cache_typing(&mut request.typing, &mut cached.typing);
	Self::update_cache_to_device(&mut request.to_device, &mut cached.to_device);
	Self::update_cache_e2ee(&mut request.e2ee, &mut cached.e2ee);
}

#[implement(Service)]
fn update_cache_account_data(request: &mut AccountData, cached: &mut AccountData) {
	some_or_sticky(&mut request.enabled, &mut cached.enabled);
	some_or_sticky(&mut request.lists, &mut cached.lists);
	some_or_sticky(&mut request.rooms, &mut cached.rooms);
}

#[implement(Service)]
fn update_cache_receipts(request: &mut Receipts, cached: &mut Receipts) {
	some_or_sticky(&mut request.enabled, &mut cached.enabled);
	some_or_sticky(&mut request.rooms, &mut cached.rooms);
	some_or_sticky(&mut request.lists, &mut cached.lists);
}

#[implement(Service)]
fn update_cache_typing(request: &mut Typing, cached: &mut Typing) {
	some_or_sticky(&mut request.enabled, &mut cached.enabled);
	some_or_sticky(&mut request.rooms, &mut cached.rooms);
	some_or_sticky(&mut request.lists, &mut cached.lists);
}

#[implement(Service)]
fn update_cache_to_device(request: &mut ToDevice, cached: &mut ToDevice) {
	some_or_sticky(&mut request.enabled, &mut cached.enabled);
}

#[implement(Service)]
fn update_cache_e2ee(request: &mut E2EE, cached: &mut E2EE) {
	some_or_sticky(&mut request.enabled, &mut cached.enabled);
}

/// load params from cache if body doesn't contain it, as long as it's allowed
/// in some cases we may need to allow an empty list as an actual value
fn list_or_sticky<T: Clone>(target: &mut Vec<T>, cached: &mut Vec<T>) {
	if !target.is_empty() {
		cached.clone_from(target);
	} else {
		target.clone_from(cached);
	}
}

fn some_or_sticky<T: Clone>(target: &mut Option<T>, cached: &mut Option<T>) {
	if let Some(target) = target {
		cached.replace(target.clone());
	} else {
		target.clone_from(cached);
	}
}

#[implement(Service)]
pub fn update_known_rooms(
	&self,
	key: &ConnectionKey,
	list_id: ListId,
	new_rooms: BTreeSet<OwnedRoomId>,
	globalsince: u64,
) {
	assert!(key.2.is_some(), "Some(conn_id) required for this call");

	let cache = self.get_connection(key);
	let mut cached = cache.lock().expect("locked");
	let list_rooms = cached.known_rooms.entry(list_id).or_default();

	for (room_id, lastsince) in list_rooms.iter_mut() {
		if !new_rooms.contains(room_id) {
			*lastsince = 0;
		}
	}

	for room_id in new_rooms {
		list_rooms.insert(room_id, globalsince);
	}
}

#[implement(Service)]
pub fn update_subscriptions(&self, key: &ConnectionKey, subscriptions: Subscriptions) {
	self.get_connection(key)
		.lock()
		.expect("locked")
		.subscriptions = subscriptions;
}

#[implement(Service)]
pub fn clear_connections(
	&self,
	user_id: Option<&UserId>,
	device_id: Option<&DeviceId>,
	conn_id: Option<&ConnectionId>,
) {
	self.connections.lock().expect("locked").retain(
		|(conn_user_id, conn_device_id, conn_conn_id), _| {
			!(user_id.is_none_or(is_equal_to!(conn_user_id))
				&& device_id.is_none_or(is_equal_to!(conn_device_id))
				&& (conn_id.is_none() || conn_id == conn_conn_id.as_ref()))
		},
	);
}

#[implement(Service)]
pub fn drop_connection(&self, key: &ConnectionKey) {
	self.connections
		.lock()
		.expect("locked")
		.remove(key);
}

#[implement(Service)]
pub fn list_connections(&self) -> Vec<ConnectionKey> {
	self.connections
		.lock()
		.expect("locked")
		.keys()
		.cloned()
		.collect()
}

#[implement(Service)]
pub fn get_connection(&self, key: &ConnectionKey) -> Arc<Mutex<Cache>> {
	self.connections
		.lock()
		.expect("locked")
		.entry(key.clone())
		.or_insert_with(|| Arc::new(Mutex::new(Cache::default())))
		.clone()
}

#[implement(Service)]
pub fn find_connection(&self, key: &ConnectionKey) -> Result<Arc<Mutex<Cache>>> {
	self.connections
		.lock()
		.expect("locked")
		.get(key)
		.cloned()
		.ok_or_else(|| err!(Request(NotFound("Connection not found."))))
}

#[implement(Service)]
pub fn is_connection_cached(&self, key: &ConnectionKey) -> bool {
	self.connections
		.lock()
		.expect("locked")
		.contains_key(key)
}

#[inline]
pub fn into_connection_key<U, D, C>(user_id: U, device_id: D, conn_id: Option<C>) -> ConnectionKey
where
	U: Into<OwnedUserId>,
	D: Into<OwnedDeviceId>,
	C: Into<ConnectionId>,
{
	(user_id.into(), device_id.into(), conn_id.map(Into::into))
}
