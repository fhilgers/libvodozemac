// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, FnArg, Pat, Type, TypeReference, Error, PatType, TypeSlice, PatIdent, ReturnType, TypePath, GenericArgument, TypeArray};
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn ffi(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let inner_name = format_ident!("{}_inner", fn_name);
    let vis = &input_fn.vis;
    let output = &input_fn.sig.output;
    let output_ty_toks = match output {
        ReturnType::Type(_, ty) => quote!{ #ty },
        ReturnType::Default => quote! { () }
    };

    let mut new_args = Vec::new();
    let mut inner_args = Vec::new();
    let mut call_args = Vec::new();
    let mut needs_unsafe = false;

    let sret = if input_fn.attrs.iter().any(|attr| attr.path().is_ident("sret")) {
        let ReturnType::Type(_, ty) = output else {
            return Error::new(output.span(), "#[sret] requires a non default return value").into_compile_error().into();
        };
        Some(quote! { result: &mut ::core::mem::MaybeUninit<#ty> })
    } else { None };

    for argument in &input_fn.sig.inputs {
        let FnArg::Typed(PatType { attrs, ty, pat, .. }) = argument else {
            return Error::new(argument.span(), "#[ffi] can not be used on methods").into_compile_error().into();
        };
        let should_expand = attrs.iter().any(|attr| attr.path().is_ident("expand"));

        if !should_expand {
            new_args.push(quote! { #pat: #ty });
            inner_args.push(quote! { #pat: #ty });
            call_args.push(quote! { #pat });
            continue
        }

        needs_unsafe = true;

        let Type::Reference(TypeReference { elem, mutability, .. }) = &**ty else {
            return Error::new(ty.span(), "#[expand] can only be used on slices").into_compile_error().into();
        };
        let Type::Slice(TypeSlice { elem, .. } ) = &**elem else {
            return Error::new(ty.span(), "#[expand] can only be used on slices").into_compile_error().into();
        };

        let Pat::Ident(PatIdent { ident, .. }) = &**pat else {
            return Error::new(ty.span(), "#[expand] can only be used on named types").into_compile_error().into();
        };

        let ptr_ident = format_ident!("{}_ptr", ident);
        let len_ident = format_ident!("{}_len", ident);

        let ptr_type = if mutability.is_some() {
            quote!(*mut #elem)
        } else {
            quote!(*const #elem)
        };

        new_args.push(quote! { #ptr_ident: #ptr_type, #len_ident: u32 });
        inner_args.push(quote! { #pat: &#mutability [#elem] });

        let slice_expr = if mutability.is_some() {
            quote! {
                {
                    if #len_ident == 0 {
                        &mut []
                    } else {
                        unsafe {
                            &mut *::core::slice::from_raw_parts_mut(#ptr_ident, #len_ident as usize)
                        }
                    }
                }
            }
        } else {
            quote! {
                {
                    if #len_ident == 0 {
                        &[]
                    } else {
                        unsafe {
                            &*::core::slice::from_raw_parts(#ptr_ident, #len_ident as usize)
                        }
                    }
                }
            }
        };

        call_args.push(slice_expr);
        continue;
    }

    let block = &input_fn.block;
    let unsafe_ident = if needs_unsafe {
        Some(quote!(unsafe))
    } else {
        None
    };

    let inner_fn = quote!{
        fn #inner_name(#(#inner_args),*) #output #block
    };

    let expanded = match &sret {
        Some(arg) => {
            quote! {
                #[unsafe(no_mangle)]
                #vis #unsafe_ident extern "C" fn #fn_name(#arg,#(#new_args),*) {
                    #inner_fn
                    result.write(#inner_name(#(#call_args),*));
                }
            }
        },
        None => {
            quote! {
                #[unsafe(no_mangle)]
                #vis #unsafe_ident extern "C" fn #fn_name(#(#new_args),*) #output {
                    #inner_fn
                    #inner_name(#(#call_args),*)
                }
            }
        }
    };

    let jni_name = format_ident!("{}_JNI", fn_name.to_string().to_uppercase());


    let mut pre_statements = Vec::new();
    let mut post_statements = Vec::new();
    let mut jni_signature = String::new();
    let mut jni_args = Vec::new();
    let mut jni_call_args = Vec::new();
    let mut jni_raw_call_args = Vec::new();

    for argument in &input_fn.sig.inputs {
        let FnArg::Typed(PatType { ty, pat, .. }) = argument else {
            return Error::new(argument.span(), "#[ffi] can not be used on methods").into_compile_error().into();
        };

        if let Type::Reference(TypeReference { elem, mutability, .. }) = &**ty {
            if let Type::Slice(TypeSlice { elem, .. }) = &**elem {
                let Pat::Ident(PatIdent { ident, .. }) = &**pat else {
                    return Error::new(ty.span(), "#[expand] can only be used on named types").into_compile_error().into();
                };
                let Type::Path(TypePath { path, .. }) = &**elem else {
                    return Error::new(elem.span(), "#[ffi] only supports primitive slices").into_compile_error().into();
                };
                if path.segments.len() != 1 {
                    return Error::new(path.span(), "#[ffi] only supports primitive slices").into_compile_error().into();
                }
                let segment = path.segments.get(0).expect("first elem to exist after size check");

                let arg_type = match segment.ident.to_string().as_str() {
                    "i8" | "u8" => quote! { ::jni::objects::JByteArray },
                    "i16" | "u16" => quote! { ::jni::objects::JShortArray },
                    "i32" | "u32" => quote! { ::jni::objects::JIntArray },
                    "i64" | "u64" => quote! { ::jni::objects::JLongArray },
                    "isize" | "usize" => quote! { ::jni::objects::JLongArray },
                    _ => {
                        return Error::new(ident.span(), "#[ffi] only supports primitive slices").into_compile_error().into();
                    }
                };

                let size_ident = format_ident!("{}_len", ident);
                pre_statements.push(quote! { let #ident = crate::get_byte_array_region(env, &#ident, 0, #size_ident)?; });
                jni_call_args.push(quote! { #ident.as_ptr(), #size_ident });
                jni_raw_call_args.push(quote! { #ident, #size_ident });
                jni_args.push(quote! { #ident: #arg_type, #size_ident: u32 });
                jni_signature.push_str("Ljava/lang/Object;");
                jni_signature.push_str("I");
                continue;
            } else if let Type::Array(TypeArray { elem, len, .. }) = &**elem {
                let Pat::Ident(PatIdent { ident, .. }) = &**pat else {
                    return Error::new(ty.span(), "#[expand] can only be used on named types").into_compile_error().into();
                };
                let Type::Path(TypePath { path, .. }) = &**elem else {
                    return Error::new(elem.span(), "#[ffi] only supports primitive slices").into_compile_error().into();
                };
                if path.segments.len() != 1 {
                    return Error::new(path.span(), "#[ffi] only supports primitive slices").into_compile_error().into();
                }
                let segment = path.segments.get(0).expect("first elem to exist after size check");

                let native_ident = format_ident!("{ident}_native");

                let arg_type = match segment.ident.to_string().as_str() {
                    "i8" | "u8" => {
                        if mutability.is_some() {
                            pre_statements.push(quote! { let mut #native_ident = [0u8; #len]; });
                            jni_call_args.push(quote! { &mut #native_ident });
                            post_statements.push(quote! {
                                env.set_byte_array_region(#ident, 0, unsafe { &*(&#native_ident[..] as *const [u8] as *const [i8]) })?;
                            });
                        } else {
                            pre_statements.push(quote! { let #native_ident = crate::get_byte_array_region_const::<#len>(env, &#ident, 0)?; });
                            jni_call_args.push(quote! { &#native_ident });
                        }
                        quote! { ::jni::objects::JByteArray }
                    },
                    "i16" | "u16" => {
                        if mutability.is_some() {
                            pre_statements.push(quote! { let mut #native_ident = [0u16; #len]; });
                            jni_call_args.push(quote! { &mut #native_ident });
                            post_statements.push(quote! {
                                env.set_short_array_region(#ident, 0, unsafe { &*(&#native_ident[..] as *const [u16] as *const [i16]) })?;
                            });
                        } else {
                            pre_statements.push(quote! { let #native_ident = crate::get_short_array_region_const::<#len>(env, &#ident, 0)?; });
                            jni_call_args.push(quote! { &#native_ident });
                        }
                        quote! { ::jni::objects::JShortArray }
                    },
                    "i32" | "u32" => {
                        if mutability.is_some() {
                            pre_statements.push(quote! { let mut #native_ident = [0u32; #len]; });
                            jni_call_args.push(quote! { &mut #native_ident });
                            post_statements.push(quote! {
                                env.set_short_array_region(#ident, 0, unsafe { &*(&#native_ident[..] as *const [u32] as *const [i32]) })?;
                            });
                        } else {
                            pre_statements.push(quote! { let #native_ident = crate::get_int_array_region_const::<#len>(env, &#ident, 0)?; });
                            jni_call_args.push(quote! { &#native_ident });
                        }
                        quote! { ::jni::objects::JIntArray }
                    },
                    _ => {
                        return Error::new(ident.span(), "#[ffi] only supports primitive slices").into_compile_error().into();
                    }
                };
                jni_raw_call_args.push(quote! { #ident });
                jni_args.push(quote! { #ident: #arg_type });
                jni_signature.push_str("Ljava/lang/Object;");
                continue;
            } else {
                jni_signature.push_str("J");
            }
        } else if let Type::Ptr(_) = &**ty {
            jni_signature.push_str("J");
        } else if let Type::Path(TypePath { path, .. }) = &**ty {
            if path.segments.len() != 1 {
                return Error::new(path.span(), "#[ffi] does not support fully specified paths").into_compile_error().into();
            }
            let segment = path.segments.get(0).expect("first elem to exist after size check");

            let sig = match segment.ident.to_string().as_str() {
                "i8" | "u8" => "B",
                "i16" | "u16" => "S",
                "i32" | "u32" => "I",
                "i64" | "u64" => "J",
                "isize" | "usize" => "J",
                "NonNull" => "J",
                "Option" => {
                    let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
                        return Error::new(segment.ident.span(), "#[ffi] does not support this type").into_compile_error().into();
                    };
                    let GenericArgument::Type(ty) = args.args.iter().next().unwrap() else {
                        return Error::new(args.span(), "#[ffi] does not support this type").into_compile_error().into();
                    };

                    match ty {
                        Type::Path(TypePath { path, .. }) => {
                            if path.segments.len() != 1 {
                                return Error::new(path.span(), "#[ffi] does not support this type").into_compile_error().into();
                            }

                            let segment = path.segments.iter().next().unwrap();
                            if segment.ident.to_string().as_str() == "NonNull" {
                                "J"
                            } else {
                                return Error::new(segment.ident.span(), "#[ffi] does not support this type").into_compile_error().into();
                            }
                        }
                        Type::Reference(_) => {
                            "J"
                        }
                        _ => {
                            return Error::new(ty.span(), "#[ffi] does not support this type").into_compile_error().into();
                        }
                    }

                },
                _ => {
                    return Error::new(segment.ident.span(), "#[ffi] does not support this type").into_compile_error().into();
                }
            };

            jni_signature.push_str(sig);
        } else {
            return Error::new(ty.span(), "#[ffi] does not support this type").into_compile_error().into();
        }

        jni_raw_call_args.push(quote! { #pat });
        jni_call_args.push(quote! { #pat });
        jni_args.push(quote! { #pat: #ty });
    }


    let jni_output = match &sret {
        Some(_) => quote!{ -> ::jni::errors::Result<()> },
        None => quote! { -> ::jni::errors::Result<#output_ty_toks> }
    };

    let wrapped_jni_output = match &sret {
        Some(_) => Some(quote! { -> () }),
        None => Some(quote !{ -> #output_ty_toks }),
    };

    let sret_arg = match sret {
        Some(_) => Some(quote!{ result: ::jni::objects::JLongArray, }),
        None => None,
    };
    let sret_arg_call = match sret {
        Some(_) => Some(quote! {result,}),
        None => None,
    };

    let jni_ret_signature = match sret {
        Some(_) => "V",
        None => match output {
            ReturnType::Default => "V",
            ReturnType::Type(_, ty) => match &**ty {
                Type::Path(TypePath { path, ..}) => match path.segments.iter().next().unwrap().ident.to_string().as_str() {
                    "Option" | "NonNull" => "J",
                    "u32" | "i32" => "I",
                    "u64" | "i64" => "J",
                    "usize" | "isize" => "J",
                    "u16" | "i16" => "S",
                    "u8" | "i8" => "B",
                    _ => return Error::new(path.span(), "unsupported").into_compile_error().into()
                }
                Type::Ptr(_) => "J",
                _ => return Error::new(ty.span(), "unsupported").into_compile_error().into()
            }
        }
    };

    let jni_first_arg = match sret {
        Some(_) => "Ljava/lang/Object;",
        None => ""
    };

    let sret_fn_call = match sret {
        Some(_) => quote! {
            #(#pre_statements)*

            use crate::{AsUsize, CollectIntoArray};
            let lvalues = {
                let mut result = ::core::mem::MaybeUninit::uninit();
                unsafe { #fn_name(&mut result, #(#jni_call_args),*); }
                unsafe { result.assume_init() }
            }
                .as_usize()
                .map(::jni::sys::jlong::try_from)
                .map(::core::result::Result::unwrap)
                .collect_into_array::<RESULT_SIZE>();

            #(#post_statements)*

            env.set_long_array_region(result, 0, &lvalues)
        },
        None => quote! {
            #(#pre_statements)*
            let full_result = unsafe { #fn_name(#(#jni_call_args),*) };
            #(#post_statements)*
            Ok(full_result)
        }
    };

    let jni_signature = format!("({jni_first_arg}{jni_signature}){jni_ret_signature}");
    let raw_fn_name = fn_name.to_string();

    let jni = quote! {
        const #jni_name: crate::NativeMethod = {
            const RESULT_SIZE: usize =
                ::core::mem::size_of::<#output_ty_toks>() / ::core::mem::size_of::<usize>();

            #[allow(clippy::too_many_arguments)]
            fn inner(
                env: &mut ::jni::JNIEnv,
                class: &mut ::jni::objects::JClass,
                #sret_arg
                #(#jni_args),*
            ) #jni_output {
                #sret_fn_call
            }

            #[allow(clippy::too_many_arguments)]
            extern "system" fn inner_wrapped(
                mut env: ::jni::JNIEnv,
                mut class: ::jni::objects::JClass,
                #sret_arg
                #(#jni_args),*
            ) #wrapped_jni_output {
                let e = match inner(&mut env, &mut class, #sret_arg_call #(#jni_raw_call_args),*) {
                    Ok(v) => return v,
                    Err(e) => e
                };

                if matches!(env.exception_check(), Ok(false)) {
                    let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
                }

                unsafe { ::core::mem::MaybeUninit::uninit().assume_init() }
            }

            crate::NativeMethod {
                name: #raw_fn_name,
                sig: #jni_signature,
                fn_ptr: inner_wrapped as _,
            }
        };
    };

    let full_expansion = quote! {
        #expanded
        #jni
    };

    full_expansion.into()
}

/*


fn vodozemac_megolm_message_from_bytes_jni(class: &str) -> NativeMethod {
    const RESULT_SIZE: usize =
        size_of::<MaybeUninit<CResult<NonNull<MegolmMessage>, CErrorStr>>>() / size_of::<usize>();

    fn inner(
        env: &mut JNIEnv,
        class: &mut JClass,
        result: JLongArray,
        bytes: JByteArray,
        bytes_size: jint,
    ) -> jni::errors::Result<()> {
        let bytes = get_byte_array_region(env, &bytes, 0, bytes_size)?;
        let c_result = {
            let mut result = MaybeUninit::uninit();
            vodozemac_megolm_message_from_bytes(&mut result, bytes.as_ptr(), bytes_size as _);
            unsafe { result.assume_init() }
        };
        let c_result_values = c_result
            .as_usize()
            .map(jlong::try_from)
            .map(Result::unwrap)
            .collect_into_array::<RESULT_SIZE>();

        env.set_long_array_region(result, 0, &c_result_values)
    }

    fn inner_wrapped(
        mut env: JNIEnv,
        mut class: JClass,
        result: JLongArray,
        bytes: JByteArray,
        bytes_size: jint,
    ) {
        let Err(e) = inner(&mut env, &mut class, result, bytes, bytes_size) else {
            return;
        };
        if !env
            .exception_check()
            .expect("checking for an exception should not fail")
        {
            return;
        };

        env.throw_new(class, e.to_string())
            .expect("throwing an exception should not fail");
    }

    NativeMethod {
        name: format!("{class}_vodozemac_1megolm_1message_1from_1bytes").into(),
        sig: "(Ljava/lang/Object;Ljava/lang/Object;I)V".into(),
        fn_ptr: inner_wrapped as _,
    }
}
 */