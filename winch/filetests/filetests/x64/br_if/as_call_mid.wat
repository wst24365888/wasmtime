;;! target = "x86_64"
(module
  (func $f (param i32 i32 i32) (result i32) (i32.const -1)) 
  (func (export "as-call-mid") (result i32)
    (block (result i32)
      (call $f
        (i32.const 1) (br_if 0 (i32.const 13) (i32.const 1)) (i32.const 3)
      )
    )
  )
)
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4c8b5f08             	mov	r11, qword ptr [rdi + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4981c320000000       	add	r11, 0x20
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f8729000000         	ja	0x44
;;   1b:	 4989fe               	mov	r14, rdi
;;      	 4883ec20             	sub	rsp, 0x20
;;      	 48897c2418           	mov	qword ptr [rsp + 0x18], rdi
;;      	 4889742410           	mov	qword ptr [rsp + 0x10], rsi
;;      	 8954240c             	mov	dword ptr [rsp + 0xc], edx
;;      	 894c2408             	mov	dword ptr [rsp + 8], ecx
;;      	 4489442404           	mov	dword ptr [rsp + 4], r8d
;;      	 b8ffffffff           	mov	eax, 0xffffffff
;;      	 4883c420             	add	rsp, 0x20
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   44:	 0f0b                 	ud2	
;;
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4c8b5f08             	mov	r11, qword ptr [rdi + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4981c320000000       	add	r11, 0x20
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f875a000000         	ja	0x75
;;   1b:	 4989fe               	mov	r14, rdi
;;      	 4883ec10             	sub	rsp, 0x10
;;      	 48897c2408           	mov	qword ptr [rsp + 8], rdi
;;      	 48893424             	mov	qword ptr [rsp], rsi
;;      	 b901000000           	mov	ecx, 1
;;      	 b80d000000           	mov	eax, 0xd
;;      	 85c9                 	test	ecx, ecx
;;      	 0f8532000000         	jne	0x6f
;;   3d:	 4883ec04             	sub	rsp, 4
;;      	 890424               	mov	dword ptr [rsp], eax
;;      	 4883ec0c             	sub	rsp, 0xc
;;      	 4c89f7               	mov	rdi, r14
;;      	 4c89f6               	mov	rsi, r14
;;      	 ba01000000           	mov	edx, 1
;;      	 8b4c240c             	mov	ecx, dword ptr [rsp + 0xc]
;;      	 41b803000000         	mov	r8d, 3
;;      	 e800000000           	call	0x62
;;      	 4883c40c             	add	rsp, 0xc
;;      	 4883c404             	add	rsp, 4
;;      	 4c8b742408           	mov	r14, qword ptr [rsp + 8]
;;      	 4883c410             	add	rsp, 0x10
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   75:	 0f0b                 	ud2	
