<details>
<summary>XSD contract: <code>CodeSelectionType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="CodeSelectionType">
		<xs:annotation>
			<xs:documentation>CodeSelectionType defines the structure for code selection to be used as inclusive or exclusive extensions.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="MemberValue" type="MemberValueType" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>An explicit or wildcard selection of a code(s) from the codelist selected for inclusion/exclusion. If a wildcard expression is used, it is evaluated to determine codes selected for inclusion/exclusion. Otherwise, each member value is a distinct code. If the extended list is hierarchical, this can indicate whether child codes are to be included.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:sequence>
	</xs:complexType>
```

</details>
