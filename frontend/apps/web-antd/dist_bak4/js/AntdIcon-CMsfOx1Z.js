import{A as e,B as t,D as n,M as r,P as i,St as a,bt as o,k as s,t as c}from"./vue.runtime.esm-bundler-Ca66Aw3b.js";import{D as l,E as u}from"./useConfigInject-FpbH4Med.js";c();var d=Symbol(`iconContext`),f=function(){return i(d,{prefixCls:a(`anticon`),rootClassName:a(``),csp:a()})};function p(){return!!(typeof window<`u`&&window.document&&window.document.createElement)}function m(e,t){return e&&e.contains?e.contains(t):!1}var h=`data-vc-order`,g=`vc-icon-key`,_=new Map;function v(){var e=(arguments.length>0&&arguments[0]!==void 0?arguments[0]:{}).mark;return e?e.startsWith(`data-`)?e:`data-${e}`:g}function y(e){return e.attachTo?e.attachTo:document.querySelector(`head`)||document.body}function b(e){return e===`queue`?`prependQueue`:e?`prepend`:`append`}function x(e){return Array.from((_.get(e)||e).children).filter(function(e){return e.tagName===`STYLE`})}function S(e){var t=arguments.length>1&&arguments[1]!==void 0?arguments[1]:{};if(!p())return null;var n=t.csp,r=t.prepend,i=document.createElement(`style`);i.setAttribute(h,b(r)),n&&n.nonce&&(i.nonce=n.nonce),i.innerHTML=e;var a=y(t),o=a.firstChild;if(r){if(r===`queue`){var s=x(a).filter(function(e){return[`prepend`,`prependQueue`].includes(e.getAttribute(h))});if(s.length)return a.insertBefore(i,s[s.length-1].nextSibling),i}a.insertBefore(i,o)}else a.appendChild(i);return i}function C(e){var t=arguments.length>1&&arguments[1]!==void 0?arguments[1]:{};return x(y(t)).find(function(n){return n.getAttribute(v(t))===e})}function ee(e,t){var n=_.get(e);if(!n||!m(document,n)){var r=S(``,t),i=r.parentNode;_.set(e,i),e.removeChild(r)}}function te(e,t){var n=arguments.length>2&&arguments[2]!==void 0?arguments[2]:{};ee(y(n),n);var r=C(t,n);if(r)return n.csp&&n.csp.nonce&&r.nonce!==n.csp.nonce&&(r.nonce=n.csp.nonce),r.innerHTML!==e&&(r.innerHTML=e),r;var i=S(e,n);return i.setAttribute(v(n),t),i}c();function w(e){for(var t=1;t<arguments.length;t++){var n=arguments[t]==null?{}:Object(arguments[t]),r=Object.keys(n);typeof Object.getOwnPropertySymbols==`function`&&(r=r.concat(Object.getOwnPropertySymbols(n).filter(function(e){return Object.getOwnPropertyDescriptor(n,e).enumerable}))),r.forEach(function(t){T(e,t,n[t])})}return e}function T(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function E(e,t){`${t}`}function D(e){return typeof e==`object`&&typeof e.name==`string`&&typeof e.theme==`string`&&(typeof e.icon==`object`||typeof e.icon==`function`)}function O(e,t,n){return n?r(e.tag,w({key:t},n,e.attrs),(e.children||[]).map(function(n,r){return O(n,`${t}-${e.tag}-${r}`)})):r(e.tag,w({key:t},e.attrs),(e.children||[]).map(function(n,r){return O(n,`${t}-${e.tag}-${r}`)}))}function k(e){return l(e)[0]}function A(e){return e?Array.isArray(e)?e:[e]:[]}var j=`
.anticon {
  display: inline-block;
  color: inherit;
  font-style: normal;
  line-height: 0;
  text-align: center;
  text-transform: none;
  vertical-align: -0.125em;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.anticon > * {
  line-height: 1;
}

.anticon svg {
  display: inline-block;
}

.anticon::before {
  display: none;
}

.anticon .anticon-icon {
  display: block;
}

.anticon[tabindex] {
  cursor: pointer;
}

.anticon-spin::before,
.anticon-spin {
  display: inline-block;
  -webkit-animation: loadingCircle 1s infinite linear;
  animation: loadingCircle 1s infinite linear;
}

@-webkit-keyframes loadingCircle {
  100% {
    -webkit-transform: rotate(360deg);
    transform: rotate(360deg);
  }
}

@keyframes loadingCircle {
  100% {
    -webkit-transform: rotate(360deg);
    transform: rotate(360deg);
  }
}
`;function M(e){return e&&e.getRootNode&&e.getRootNode()}function N(e){return p()?M(e)instanceof ShadowRoot:!1}function P(e){return N(e)?M(e):null}var F=function(){var n=f(),r=n.prefixCls,i=n.csp,a=e(),o=j;r&&(o=o.replace(/anticon/g,r.value)),t(function(){if(p()){var e=a.vnode.el,t=P(e);te(o,`@ant-design-vue-icons`,{prepend:!0,csp:i.value,attachTo:t})}})};c();var ne=[`icon`,`primaryColor`,`secondaryColor`];function re(e,t){if(e==null)return{};var n=I(e,t),r,i;if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);for(i=0;i<a.length;i++)r=a[i],!(t.indexOf(r)>=0)&&Object.prototype.propertyIsEnumerable.call(e,r)&&(n[r]=e[r])}return n}function I(e,t){if(e==null)return{};var n={},r=Object.keys(e),i,a;for(a=0;a<r.length;a++)i=r[a],!(t.indexOf(i)>=0)&&(n[i]=e[i]);return n}function L(e){for(var t=1;t<arguments.length;t++){var n=arguments[t]==null?{}:Object(arguments[t]),r=Object.keys(n);typeof Object.getOwnPropertySymbols==`function`&&(r=r.concat(Object.getOwnPropertySymbols(n).filter(function(e){return Object.getOwnPropertyDescriptor(n,e).enumerable}))),r.forEach(function(t){R(e,t,n[t])})}return e}function R(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}var z=o({primaryColor:`#333`,secondaryColor:`#E6E6E6`,calculated:!1});function B(e){var t=e.primaryColor,n=e.secondaryColor;z.primaryColor=t,z.secondaryColor=n||k(t),z.calculated=!!n}function V(){return L({},z)}var H=function(e,t){var n=L({},e,t.attrs),r=n.icon,i=n.primaryColor,a=n.secondaryColor,o=re(n,ne),s=z;if(i&&(s={primaryColor:i,secondaryColor:a||k(i)}),E(D(r),`icon should be icon definiton, but got ${r}`),!D(r))return null;var c=r;return c&&typeof c.icon==`function`&&(c=L({},c,{icon:c.icon(s.primaryColor,s.secondaryColor)})),O(c.icon,`svg-${c.name}`,L({},o,{"data-icon":c.name,width:`1em`,height:`1em`,fill:`currentColor`,"aria-hidden":`true`}))};H.props={icon:Object,primaryColor:String,secondaryColor:String,focusable:String},H.inheritAttrs=!1,H.displayName=`IconBase`,H.getTwoToneColors=V,H.setTwoToneColors=B;function U(e,t){return J(e)||q(e,t)||G(e,t)||W()}function W(){throw TypeError(`Invalid attempt to destructure non-iterable instance.
In order to be iterable, non-array objects must have a [Symbol.iterator]() method.`)}function G(e,t){if(e){if(typeof e==`string`)return K(e,t);var n=Object.prototype.toString.call(e).slice(8,-1);if(n===`Object`&&e.constructor&&(n=e.constructor.name),n===`Map`||n===`Set`)return Array.from(e);if(n===`Arguments`||/^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n))return K(e,t)}}function K(e,t){(t==null||t>e.length)&&(t=e.length);for(var n=0,r=Array(t);n<t;n++)r[n]=e[n];return r}function q(e,t){var n=e==null?null:typeof Symbol<`u`&&e[Symbol.iterator]||e[`@@iterator`];if(n!=null){var r=[],i=!0,a=!1,o,s;try{for(n=n.call(e);!(i=(o=n.next()).done)&&(r.push(o.value),!(t&&r.length===t));i=!0);}catch(e){a=!0,s=e}finally{try{!i&&n.return!=null&&n.return()}finally{if(a)throw s}}return r}}function J(e){if(Array.isArray(e))return e}function Y(e){var t=U(A(e),2),n=t[0],r=t[1];return H.setTwoToneColors({primaryColor:n,secondaryColor:r})}function ie(){var e=H.getTwoToneColors();return e.calculated?[e.primaryColor,e.secondaryColor]:e.primaryColor}c();var ae=s({name:`InsertStyles`,setup:function(){return F(),function(){return null}}});c();var oe=[`class`,`icon`,`spin`,`rotate`,`tabindex`,`twoToneColor`,`onClick`];function se(e,t){return de(e)||ue(e,t)||le(e,t)||ce()}function ce(){throw TypeError(`Invalid attempt to destructure non-iterable instance.
In order to be iterable, non-array objects must have a [Symbol.iterator]() method.`)}function le(e,t){if(e){if(typeof e==`string`)return X(e,t);var n=Object.prototype.toString.call(e).slice(8,-1);if(n===`Object`&&e.constructor&&(n=e.constructor.name),n===`Map`||n===`Set`)return Array.from(e);if(n===`Arguments`||/^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n))return X(e,t)}}function X(e,t){(t==null||t>e.length)&&(t=e.length);for(var n=0,r=Array(t);n<t;n++)r[n]=e[n];return r}function ue(e,t){var n=e==null?null:typeof Symbol<`u`&&e[Symbol.iterator]||e[`@@iterator`];if(n!=null){var r=[],i=!0,a=!1,o,s;try{for(n=n.call(e);!(i=(o=n.next()).done)&&(r.push(o.value),!(t&&r.length===t));i=!0);}catch(e){a=!0,s=e}finally{try{!i&&n.return!=null&&n.return()}finally{if(a)throw s}}return r}}function de(e){if(Array.isArray(e))return e}function Z(e){for(var t=1;t<arguments.length;t++){var n=arguments[t]==null?{}:Object(arguments[t]),r=Object.keys(n);typeof Object.getOwnPropertySymbols==`function`&&(r=r.concat(Object.getOwnPropertySymbols(n).filter(function(e){return Object.getOwnPropertyDescriptor(n,e).enumerable}))),r.forEach(function(t){Q(e,t,n[t])})}return e}function Q(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function fe(e,t){if(e==null)return{};var n=pe(e,t),r,i;if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);for(i=0;i<a.length;i++)r=a[i],!(t.indexOf(r)>=0)&&Object.prototype.propertyIsEnumerable.call(e,r)&&(n[r]=e[r])}return n}function pe(e,t){if(e==null)return{};var n={},r=Object.keys(e),i,a;for(a=0;a<r.length;a++)i=r[a],!(t.indexOf(i)>=0)&&(n[i]=e[i]);return n}Y(u.primary);var $=function(e,t){var r,i=Z({},e,t.attrs),a=i.class,o=i.icon,s=i.spin,c=i.rotate,l=i.tabindex,u=i.twoToneColor,d=i.onClick,p=fe(i,oe),m=f(),h=m.prefixCls,g=m.rootClassName,_=(r={},Q(r,g.value,!!g.value),Q(r,h.value,!0),Q(r,`${h.value}-${o.name}`,!!o.name),Q(r,`${h.value}-spin`,!!s||o.name===`loading`),r),v=l;v===void 0&&d&&(v=-1);var y=c?{msTransform:`rotate(${c}deg)`,transform:`rotate(${c}deg)`}:void 0,b=se(A(u),2),x=b[0],S=b[1];return n(`span`,Z({role:`img`,"aria-label":o.name},p,{onClick:d,class:[_,a],tabindex:v}),[n(H,{icon:o,primaryColor:x,secondaryColor:S,style:y},null),n(ae,null,null)])};$.props={spin:Boolean,rotate:Number,icon:Object,twoToneColor:[String,Array]},$.displayName=`AntdIcon`,$.inheritAttrs=!1,$.getTwoToneColor=ie,$.setTwoToneColor=Y;export{$ as t};