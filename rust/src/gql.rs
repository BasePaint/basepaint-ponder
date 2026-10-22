// GraphQL schema generated from tables.rs the way Ponder 0.16 builds it
// (src/graphql/index.ts buildGraphQLSchema), so it is a drop-in for the TypeScript server.

use crate::query::{self, Page, PluralArgs};
use crate::state::State;
use crate::tables::{ALL, ColType, Rel, T, TableDef, defs};
use crate::value::V;
use async_graphql::Value as G;
use async_graphql::dynamic::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Db {
    pub state: RwLock<State>,
    pub defs: Vec<TableDef>,
    pub ready: std::sync::atomic::AtomicBool,
}

impl Db {
    pub fn new(state: State) -> Db {
        Db {
            state: RwLock::new(state),
            defs: defs(),
            ready: false.into(),
        }
    }

    pub fn def(&self, t: T) -> &TableDef {
        &self.defs[ALL.iter().position(|x| *x == t).unwrap()]
    }

    pub fn status(&self) -> serde_json::Value {
        let block = self.state.read().synced_to;
        serde_json::json!({ "base": { "id": 8453, "block": { "number": block, "timestamp": crate::logs::block_timestamp(block) } } })
    }
}

#[derive(Clone, Copy)]
struct RowRef {
    t: T,
    rid: u32,
}

const PLURAL_ARGS: [(&str, &str); 7] = [
    ("where", ""),
    ("orderBy", TypeRef::STRING),
    ("orderDirection", TypeRef::STRING),
    ("before", TypeRef::STRING),
    ("after", TypeRef::STRING),
    ("limit", TypeRef::INT),
    ("offset", TypeRef::INT),
];

fn with_plural_args(mut f: Field, filter: &str) -> Field {
    for (name, ty) in PLURAL_ARGS {
        f = f.argument(InputValue::new(
            name,
            TypeRef::named(if ty.is_empty() { filter } else { ty }),
        ));
    }
    f
}

fn plural_args(ctx: &ResolverContext) -> PluralArgs {
    let s = |k: &str| {
        ctx.args
            .get(k)
            .and_then(|v| v.string().ok().map(str::to_string))
    };
    PluralArgs {
        where_: ctx.args.get("where").map(|v| v.as_value().clone()),
        order_by: s("orderBy"),
        order_direction: s("orderDirection"),
        before: s("before"),
        after: s("after"),
        limit: ctx.args.get("limit").and_then(|v| v.i64().ok()),
        offset: ctx.args.get("offset").and_then(|v| v.i64().ok()),
    }
}

fn scalar_type(ty: ColType) -> &'static str {
    match ty {
        ColType::Int => TypeRef::INT,
        ColType::BigInt => "BigInt",
        ColType::Text => TypeRef::STRING,
    }
}

fn err(e: String) -> async_graphql::Error {
    async_graphql::Error::new(e)
}

fn run_plural(
    db: &Db,
    t: T,
    args: &PluralArgs,
    want_total: bool,
    extra: Option<(usize, V)>,
) -> async_graphql::Result<Page> {
    let st = db.state.read();
    query::plural(&st, db.def(t), args, want_total, extra).map_err(err)
}

const SUFFIX_UNIVERSAL: [&str; 2] = ["", "_not"];
const SUFFIX_SINGULAR: [&str; 2] = ["_in", "_not_in"];
const SUFFIX_NUMERIC: [&str; 4] = ["_gt", "_lt", "_gte", "_lte"];
const SUFFIX_STRING: [&str; 12] = [
    "_contains",
    "_not_contains",
    "_starts_with",
    "_ends_with",
    "_not_starts_with",
    "_not_ends_with",
    "_contains_nocase",
    "_not_contains_nocase",
    "_starts_with_nocase",
    "_ends_with_nocase",
    "_not_starts_with_nocase",
    "_not_ends_with_nocase",
];

pub fn build(db: Arc<Db>) -> Schema {
    let mut query = Object::new("Query");
    let mut types: Vec<Object> = Vec::new();
    let mut inputs: Vec<InputObject> = Vec::new();

    for def in &db.defs {
        let t = def.t;
        let name = def.name;
        let filter_name = format!("{name}Filter");
        let page_name = format!("{name}Page");

        // Filter input
        let mut filter = InputObject::new(&filter_name)
            .field(InputValue::new("AND", TypeRef::named_list(&filter_name)))
            .field(InputValue::new("OR", TypeRef::named_list(&filter_name)));
        for col in &def.cols {
            let ty = scalar_type(col.ty);
            for s in SUFFIX_UNIVERSAL {
                filter = filter.field(InputValue::new(
                    format!("{}{s}", col.name),
                    TypeRef::named(ty),
                ));
            }
            for s in SUFFIX_SINGULAR {
                filter = filter.field(InputValue::new(
                    format!("{}{s}", col.name),
                    TypeRef::named_list(ty),
                ));
            }
            let extra: &[&str] = if col.ty == ColType::Text {
                &SUFFIX_STRING
            } else {
                &SUFFIX_NUMERIC
            };
            for s in extra {
                filter = filter.field(InputValue::new(
                    format!("{}{s}", col.name),
                    TypeRef::named(ty),
                ));
            }
        }
        inputs.push(filter);

        // Entity type
        let mut obj = Object::new(name);
        for (ci, col) in def.cols.iter().enumerate() {
            let ty = if col.not_null {
                TypeRef::named_nn(scalar_type(col.ty))
            } else {
                TypeRef::named(scalar_type(col.ty))
            };
            obj = obj.field(Field::new(col.name, ty, move |ctx| {
                FieldFuture::new(async move {
                    let r = ctx.parent_value.try_downcast_ref::<RowRef>()?;
                    let db = ctx.data::<Arc<Db>>()?;
                    let v = db.state.read().get(r.t, r.rid, ci);
                    Ok(match v {
                        V::Null => None,
                        v => Some(FieldValue::value(v.to_gql())),
                    })
                })
            }));
        }
        for rel in &def.rels {
            match rel {
                Rel::One {
                    name: rname,
                    table,
                    local,
                    ..
                } => {
                    let (target, local_ci) = (*table, def.col(local).unwrap());
                    let target_name = db.def(target).name;
                    obj = obj.field(Field::new(
                        *rname,
                        TypeRef::named(target_name),
                        move |ctx| {
                            FieldFuture::new(async move {
                                let r = ctx.parent_value.try_downcast_ref::<RowRef>()?;
                                let db = ctx.data::<Arc<Db>>()?;
                                let st = db.state.read();
                                let key = st.get(r.t, r.rid, local_ci);
                                Ok(st
                                    .find(target, &[key])
                                    .map(|rid| FieldValue::owned_any(RowRef { t: target, rid })))
                            })
                        },
                    ));
                }
                Rel::Many {
                    name: rname,
                    table,
                    local,
                    remote,
                } => {
                    let target = *table;
                    let tdef = db.def(target);
                    let (local_ci, remote_ci) =
                        (def.col(local).unwrap(), tdef.col(remote).unwrap());
                    let f = Field::new(
                        *rname,
                        TypeRef::named(format!("{}Page", tdef.name)),
                        move |ctx| {
                            FieldFuture::new(async move {
                                let r = ctx.parent_value.try_downcast_ref::<RowRef>()?;
                                let db = ctx.data::<Arc<Db>>()?;
                                let key = db.state.read().get(r.t, r.rid, local_ci);
                                let want_total = ctx.look_ahead().field("totalCount").exists();
                                let page = run_plural(
                                    db,
                                    target,
                                    &plural_args(&ctx),
                                    want_total,
                                    Some((remote_ci, key)),
                                )?;
                                Ok(Some(FieldValue::owned_any(page)))
                            })
                        },
                    );
                    obj = obj.field(with_plural_args(f, &format!("{}Filter", tdef.name)));
                }
            }
        }
        types.push(obj);

        // Page type
        let page = Object::new(&page_name)
            .field(Field::new(
                "items",
                TypeRef::named_nn_list_nn(name),
                |ctx| {
                    FieldFuture::new(async move {
                        let p = ctx.parent_value.try_downcast_ref::<Page>()?;
                        let t = p.table;
                        Ok(Some(FieldValue::list(
                            p.rows
                                .iter()
                                .map(|&rid| FieldValue::owned_any(RowRef { t, rid })),
                        )))
                    })
                },
            ))
            .field(Field::new(
                "pageInfo",
                TypeRef::named_nn("PageInfo"),
                |ctx| {
                    FieldFuture::new(async move {
                        let p = ctx.parent_value.try_downcast_ref::<Page>()?;
                        Ok(Some(FieldValue::borrowed_any(p)))
                    })
                },
            ))
            .field(Field::new(
                "totalCount",
                TypeRef::named_nn(TypeRef::INT),
                |ctx| {
                    FieldFuture::new(async move {
                        let p = ctx.parent_value.try_downcast_ref::<Page>()?;
                        Ok(p.total.map(|n| FieldValue::value(n)))
                    })
                },
            ));
        types.push(page);

        // Singular query, keyed by the primary key columns
        let pk: Vec<(usize, &'static str, ColType)> = def
            .pk
            .iter()
            .map(|&i| (i, def.cols[i].name, def.cols[i].ty))
            .collect();
        let singular = name[..1].to_lowercase() + &name[1..];
        let pk_owned = pk.clone();
        let mut f = Field::new(&singular, TypeRef::named(name), move |ctx| {
            let pk = pk_owned.clone();
            FieldFuture::new(async move {
                let db = ctx.data::<Arc<Db>>()?;
                let mut key = Vec::new();
                for (_, cname, ty) in &pk {
                    key.push(V::from_gql(ctx.args.try_get(cname)?.as_value(), *ty).map_err(err)?);
                }
                Ok(db
                    .state
                    .read()
                    .find(t, &key)
                    .map(|rid| FieldValue::owned_any(RowRef { t, rid })))
            })
        });
        for (_, cname, ty) in &pk {
            // Ponder types composite primary key arguments as Float.
            let ty = if pk.len() > 1 && *ty == ColType::Int {
                TypeRef::FLOAT
            } else {
                scalar_type(*ty)
            };
            f = f.argument(InputValue::new(*cname, TypeRef::named_nn(ty)));
        }
        query = query.field(f);

        let f = Field::new(
            format!("{singular}s"),
            TypeRef::named_nn(&page_name),
            move |ctx| {
                FieldFuture::new(async move {
                    let db = ctx.data::<Arc<Db>>()?;
                    let want_total = ctx.look_ahead().field("totalCount").exists();
                    let page = run_plural(db, t, &plural_args(&ctx), want_total, None)?;
                    Ok(Some(FieldValue::owned_any(page)))
                })
            },
        );
        query = query.field(with_plural_args(f, &filter_name));
    }

    query = query.field(Field::new("_meta", TypeRef::named("Meta"), |ctx| {
        FieldFuture::new(async move {
            let db = ctx.data::<Arc<Db>>()?;
            Ok(Some(FieldValue::owned_any(db.status())))
        })
    }));

    let page_info = |name: &str, cursors: bool| {
        let b = |field: &'static str| {
            Field::new(field, TypeRef::named_nn(TypeRef::BOOLEAN), move |ctx| {
                FieldFuture::new(async move {
                    let p = ctx.parent_value.try_downcast_ref::<Page>()?;
                    Ok(Some(FieldValue::value(if field == "hasNextPage" {
                        p.has_next
                    } else {
                        p.has_prev
                    })))
                })
            })
        };
        let mut o = Object::new(name)
            .field(b("hasNextPage"))
            .field(b("hasPreviousPage"));
        if cursors {
            for field in ["startCursor", "endCursor"] {
                o = o.field(Field::new(
                    field,
                    TypeRef::named(TypeRef::STRING),
                    move |ctx| {
                        FieldFuture::new(async move {
                            let p = ctx.parent_value.try_downcast_ref::<Page>()?;
                            let c = if field == "startCursor" {
                                &p.start_cursor
                            } else {
                                &p.end_cursor
                            };
                            Ok(c.clone().map(|c| FieldValue::value(c)))
                        })
                    },
                ));
            }
        }
        o
    };

    let meta = Object::new("Meta").field(Field::new("status", TypeRef::named("JSON"), |ctx| {
        FieldFuture::new(async move {
            let s = ctx.parent_value.try_downcast_ref::<serde_json::Value>()?;
            Ok(Some(FieldValue::value(G::from_json(s.clone())?)))
        })
    }));

    let mut schema = Schema::build("Query", None, None)
        .register(Scalar::new("JSON").description(
            "The `JSON` scalar type represents JSON values as specified by [ECMA-404](http://www.ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf).",
        ))
        .register(Scalar::new("BigInt"))
        .register(page_info("PageInfo", true))
        .register(page_info("ViewPageInfo", false))
        .register(meta)
        .register(query)
        .data(db.clone())
        .limit_depth(100);
    for o in types {
        schema = schema.register(o);
    }
    for i in inputs {
        schema = schema.register(i);
    }
    schema.finish().expect("valid GraphQL schema")
}
