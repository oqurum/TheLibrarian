use yew::prelude::*;

use crate::is_signed_in;


#[derive(Properties, PartialEq)]
pub struct LoginBarrierProps {
    pub children: Children,
}


#[function_component(LoginBarrier)]
pub fn login_barrier(props: &LoginBarrierProps) -> Html {
    if is_signed_in() {
        html! {
            for props.children.iter()
        }
    } else {
        html! {}
    }
}