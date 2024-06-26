;;! target = "x86_64"
;;!
;;! settings = ['enable_heap_access_spectre_mitigation=true']
;;!
;;! compile = false
;;!
;;! [globals.vmctx]
;;! type = "i64"
;;! vmctx = true
;;!
;;! [globals.heap_base]
;;! type = "i64"
;;! load = { base = "vmctx", offset = 0, readonly = true }
;;!
;;! [globals.heap_bound]
;;! type = "i64"
;;! load = { base = "vmctx", offset = 8, readonly = true }
;;!
;;! [[heaps]]
;;! base = "heap_base"
;;! min_size = 0x10000
;;! offset_guard_size = 0
;;! index_type = "i32"
;;! style = { kind = "dynamic", bound = "heap_bound" }

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0x1000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0x1000))

;; function u0:0(i32, i32, i64 vmctx) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned readonly gv0
;;
;;                                 block0(v0: i32, v1: i32, v2: i64):
;; @0040                               v3 = uextend.i64 v0
;; @0040                               v4 = global_value.i64 gv1
;; @0040                               v5 = iconst.i64 4097
;; @0040                               v6 = isub v4, v5  ; v5 = 4097
;; @0040                               v7 = icmp ugt v3, v6
;; @0040                               v8 = global_value.i64 gv2
;; @0040                               v9 = iadd v8, v3
;; @0040                               v10 = iconst.i64 4096
;; @0040                               v11 = iadd v9, v10  ; v10 = 4096
;; @0040                               v12 = iconst.i64 0
;; @0040                               v13 = select_spectre_guard v7, v12, v11  ; v12 = 0
;; @0040                               istore8 little heap v1, v13
;; @0044                               jump block1
;;
;;                                 block1:
;; @0044                               return
;; }
;;
;; function u0:1(i32, i64 vmctx) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned readonly gv0
;;
;;                                 block0(v0: i32, v1: i64):
;; @0049                               v3 = uextend.i64 v0
;; @0049                               v4 = global_value.i64 gv1
;; @0049                               v5 = iconst.i64 4097
;; @0049                               v6 = isub v4, v5  ; v5 = 4097
;; @0049                               v7 = icmp ugt v3, v6
;; @0049                               v8 = global_value.i64 gv2
;; @0049                               v9 = iadd v8, v3
;; @0049                               v10 = iconst.i64 4096
;; @0049                               v11 = iadd v9, v10  ; v10 = 4096
;; @0049                               v12 = iconst.i64 0
;; @0049                               v13 = select_spectre_guard v7, v12, v11  ; v12 = 0
;; @0049                               v14 = uload8.i32 little heap v13
;; @004d                               jump block1(v14)
;;
;;                                 block1(v2: i32):
;; @004d                               return v2
;; }
